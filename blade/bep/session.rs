use std::sync::Arc;

use build_event_stream_proto::*;
use build_proto::google::devtools::build::v1::*;
use lazy_static::lazy_static;
use prometheus_client::metrics::counter::Counter;
use prost_reflect::prost::Message;

use crate::{BuildState, EventHandler, ProccessedEvent};

lazy_static! {
    static ref MESSAGE_HANDLER_ERRORS: Counter::<u64> = metrics::register_metric(
        "blade_bep_message_handler_errors",
        "Total number of errors returned by the message handlers",
        Counter::default()
    );
}

pub(crate) struct BESSession {
    handlers: Arc<Vec<Box<dyn EventHandler + Sync + Send>>>,
    global: Arc<state::Global>,
    invocation_id: String,
    build_over: bool,
}

impl BESSession {
    pub fn new(
        handlers: Arc<Vec<Box<dyn EventHandler + Sync + Send>>>,
        global: Arc<state::Global>,
    ) -> Self {
        BESSession {
            handlers,
            global,
            invocation_id: "".to_string(),
            build_over: false,
        }
    }

    pub fn invocation_id(&self) -> &str { &self.invocation_id }

    pub async fn process_message(
        &mut self,
        msg: Option<PublishBuildToolEventStreamRequest>,
    ) -> Result<crate::ProccessedEvent, tonic::Status> {
        let Some(msg) = msg else {
            return Ok(ProccessedEvent {
                obe: OrderedBuildEvent::default(),
            });
        };
        if self.invocation_id.is_empty() {
            self.invocation_id = extract_session_id(&msg)?;
            let span = tracing::span::Span::current();
            span.record("session_uuid", &self.invocation_id);
            tracing::info!("Stream started");
            validate_stream(self.global.clone(), &self.invocation_id).await?;
            create_invocation(self.global.db_manager.clone(), &self.invocation_id)
                .await
                .map_err(|e| tonic::Status::internal(format!("{e:#?}")))?;
        }

        // Update heartbeat for liveness tracking
        let mgr = self.global.db_manager.clone();
        let inv_id = self.invocation_id.to_string();
        let _ = db::run(mgr, move |db| {
            db.update_invocation_heartbeat(&inv_id)
        })
        .await
        .inspect_err(|e| {
            tracing::warn!(
                "Failed to update heartbeat for {}: {:#?}",
                self.invocation_id,
                e
            );
        });

        let Some(obe) = msg.ordered_build_event else {
            return Err(tonic::Status::invalid_argument("Empty OBE"));
        };
        let state = self.handle_ordered_build_event(&obe).await?;
        matches!(state, BuildState::BuildFinished).then(|| {
            self.build_over = true;
        });
        Ok(crate::ProccessedEvent { obe })
    }

    async fn handle_ordered_build_event(
        &self,
        obe: &OrderedBuildEvent,
    ) -> Result<BuildState, tonic::Status> {
        let mut build_state = BuildState::BuildInProgress;
        let Some(event) = obe.event.as_ref().and_then(|event| event.event.as_ref()) else {
            // If there is no event for some reason, just read the next event.
            return Ok(BuildState::BuildInProgress);
        };
        match event {
            build_event::Event::BazelEvent(any) => {
                let be = build_event_stream::BuildEvent::decode(&any.value[..]).map_err(|e| {
                    tonic::Status::invalid_argument(format!("badly formatted BuildEvent: {e:#?}"))
                })?;
                be.last_message.then(|| {
                    build_state = BuildState::BuildFinished;
                });
                match be.payload.as_ref() {
                    Some(build_event_stream::build_event::Payload::Finished(f)) => {
                        let success = f
                            .exit_code
                            .as_ref()
                            .unwrap_or(&build_event_stream::build_finished::ExitCode {
                                name: "idk".into(),
                                code: 1,
                            })
                            .code
                            == 0;
                        write_session_result(
                            self.global.db_manager.clone(),
                            &self.invocation_id,
                            success,
                        )
                        .await
                        .map_err(|e| tonic::Status::internal(format!("{e:#?}")))?;
                    },
                    Some(_) => {
                        for v in &*self.handlers {
                            if let Err(e) =
                                v.handle_event(&*self.global.db_manager, &self.invocation_id, &be)
                            {
                                tracing::warn!("{:#?}", e);
                                MESSAGE_HANDLER_ERRORS.inc();
                            }
                        }
                    },
                    _ => {},
                }
            },
            build_event::Event::ComponentStreamFinished(_) => {
                build_state = BuildState::BuildFinished;
            },
            _ => {},
        }
        Ok(build_state)
    }

    pub fn is_build_over(&self) -> bool { self.build_over }
}

fn extract_session_id(
    msg: &build_proto::google::devtools::build::v1::PublishBuildToolEventStreamRequest,
) -> Result<String, tonic::Status> {
    let id = msg
        .ordered_build_event
        .as_ref()
        .and_then(|obe| obe.stream_id.as_ref())
        .map(|sid| sid.invocation_id.clone())
        .ok_or_else(|| tonic::Status::invalid_argument("Missing stream id"))?;
    Ok(id)
}

async fn validate_stream(global: Arc<state::Global>, session_uuid: &str) -> Result<(), tonic::Status> {
    let mgr = global.db_manager.clone();
    let session_id = session_uuid.to_string();
    let session_lock_time = global.session_lock_time;

    let inv = db::run(mgr, move |db| {
        db.get_shallow_invocation(&session_id)
    })
    .await
    .map_err(|e| tonic::Status::not_found(format!("{e:#?}")))?;

    if let Some(end) = inv.end
        && std::time::SystemTime::now()
            .duration_since(end)
            .unwrap_or(std::time::Duration::from_secs(0))
            > session_lock_time
    {
        return Err(tonic::Status::failed_precondition("session already ended"));
    }
    Ok(())
}

async fn write_session_result(
    db_mgr: std::sync::Arc<dyn state::DBManager>,
    invocation_id: &str,
    success: bool,
) -> anyhow::Result<()> {
    let inv_id = invocation_id.to_string();
    db::run_group(db_mgr, move |db| {
        db.update_shallow_invocation(
            &inv_id,
            Box::new(move |i: &mut state::InvocationResults| {
                match success {
                    true => i.status = state::Status::Success,
                    false => i.status = state::Status::Fail,
                }
                i.end = Some(std::time::SystemTime::now());
                Ok(())
            }),
        )
    })
    .await
}

async fn create_invocation(db_mgr: std::sync::Arc<dyn state::DBManager>, invocation_id: &str) -> anyhow::Result<()> {
    let inv_id = invocation_id.to_string();
    db::run_group(db_mgr, move |db| {
        db.upsert_shallow_invocation(&state::InvocationResults {
            id: inv_id,
            status: state::Status::InProgress,
            start: std::time::SystemTime::now(),
            ..Default::default()
        })
    })
    .await
}
