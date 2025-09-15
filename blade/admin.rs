use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use actix_web::*;
use anyhow::Context;
use cfg_if::cfg_if;
use futures::prelude::future::FutureExt;
use tracing::instrument;

#[instrument]
pub async fn run_admin_server(
    admin_host: SocketAddr,
    filter_channel: tokio::sync::mpsc::Sender<String>,
    span_channel: tokio::sync::mpsc::Sender<bool>,
    re_handle: Arc<Mutex<regex::Regex>>,
) -> anyhow::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(filter_channel.clone()))
            .app_data(web::Data::new(span_channel.clone()))
            .app_data(web::Data::new(re_handle.clone()))
            .service(set_filter)
            .service(set_span)
            .service(metrics_handler)
            .service(debug_message_handler)
            .service(debug_mem_stats_handler)
            .service(debug_mem_profile_handler)
            .service(debug_mem_profile_enable_handler)
            .service(debug_stackz_handler)
            .wrap(tracing_actix_web::TracingLogger::<
                super::BladeRootSpanBuilder,
            >::new())
    })
    .disable_signals()
    .bind(&admin_host)?
    .run()
    .map(|_| ())
    .await;
    Ok(())
}

#[post("/admin/log_filter")]
#[instrument]
async fn set_filter(
    filter_channel: web::Data<tokio::sync::mpsc::Sender<String>>,
    body: String,
) -> impl Responder {
    filter_channel
        .send(body)
        .await
        .context("failed to send filter")
        .map_err(|e| error::ErrorInternalServerError(format!("{e}")))?;
    HttpResponse::Ok().await
}

#[post("/admin/span")]
#[instrument]
async fn set_span(
    span_channel: web::Data<tokio::sync::mpsc::Sender<bool>>,
    body: String,
) -> impl Responder {
    let enable = body
        .trim()
        .parse::<bool>()
        .context("failed to parse span enable")
        .map_err(|e| error::ErrorInternalServerError(format!("{e}")))?;
    span_channel
        .send(enable)
        .await
        .context("failed to send span enable")
        .map_err(|e| error::ErrorInternalServerError(format!("{e}")))?;
    HttpResponse::Ok().await
}

#[get("/admin/metrics")]
#[instrument]
async fn metrics_handler() -> Result<HttpResponse> {
    let body = metrics::openmetrics_string()
        .map_err(|e| error::ErrorInternalServerError(format!("{e}")))?;
    Ok(HttpResponse::Ok()
        .content_type("application/openmetrics-text; version=1.0.0; charset=utf-8")
        .body(body))
}

#[post("/admin/debug_message")]
#[instrument(skip(re_handle))]
async fn debug_message_handler(
    re_handle: web::Data<Arc<Mutex<regex::Regex>>>,
    body: String,
) -> Result<HttpResponse> {
    {
        let mut re = re_handle.lock().unwrap();
        *re = regex::Regex::new(&body)
            .map_err(|e| error::ErrorInternalServerError(format!("{e}")))?;
    }
    HttpResponse::Ok().await
}

#[get("/admin/mem/stats")]
#[instrument]
async fn debug_mem_stats_handler() -> Result<HttpResponse> {
    let buf = memdump::stats().await.map_err(|e| {
        error::ErrorInternalServerError(format!("error getting memdump stats: {e:#?}"))
    })?;
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(buf))
}

#[get("/admin/mem/dump")]
#[instrument]
async fn debug_mem_profile_handler() -> Result<HttpResponse> {
    let buf = memdump::dump_profile().await.map_err(|e| {
        error::ErrorInternalServerError(format!("error getting memdump profile: {e:#?}"))
    })?;

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .body(buf))
}

#[derive(Debug, serde::Deserialize)]
struct MemdumpEnableQuery {
    enable: bool,
}

#[get("/admin/mem/enable")]
#[instrument]
async fn debug_mem_profile_enable_handler(
    info: web::Query<MemdumpEnableQuery>,
) -> Result<HttpResponse> {
    memdump::enable_profiling(info.enable).await.map_err(|e| {
        error::ErrorInternalServerError(format!("error setting memdump status: {e:#?}"))
    })?;
    Ok(HttpResponse::Ok().into())
}

#[get("/admin/stackz")]
#[instrument]
async fn debug_stackz_handler() -> Result<HttpResponse> {
    cfg_if! {
        if #[cfg(not(target_os = "linux"))] {
            return Err(error::ErrorInternalServerError("rstack-self not supported on non-Linux"));
        } else {
            let exe = std::env::current_exe().map_err(|e| error::ErrorInternalServerError(format!("error current bin: {e:#?}")))?.into_os_string();
            let trace = rstack_self::trace(std::process::Command::new(exe).arg("--rstack_child")).map_err(|e| error::ErrorInternalServerError(format!("error getting stack trace: {e:#?}")))?;
            Ok(HttpResponse::Ok().content_type("text/plain").body(format_stacktrace(&trace)))
        }
    }
}

cfg_if! {
if #[cfg(target_os = "linux")] {
use rstack_self::Trace;
use std::fmt::Write;

/// Format a stack trace into a nice string for HTTP responses
fn format_stacktrace(trace: &Trace) -> String {
    let mut output = String::new();

    let threads = trace.threads();
    if threads.is_empty() {
        return "No threads found in trace".to_string();
    }

    writeln!(output, "Stack Trace ({} threads)", threads.len()).unwrap();
    writeln!(output, "{}", "=".repeat(60)).unwrap();

    for (thread_idx, thread) in threads.iter().enumerate() {
        if thread_idx > 0 {
            writeln!(output).unwrap(); // Blank line between threads
        }

        // Thread header
        writeln!(output, "Thread {} ({})", thread.id(), thread.name()).unwrap();
        writeln!(output, "{}", "-".repeat(40)).unwrap();

        let frames = thread.frames();
        if frames.is_empty() {
            writeln!(output, "  No stack frames").unwrap();
            continue;
        }

        for (frame_idx, frame) in frames.iter().enumerate() {
            writeln!(output, "  #{:2} 0x{:016x}", frame_idx, frame.ip()).unwrap();

            let symbols = frame.symbols();
            if symbols.is_empty() {
                writeln!(output, "      [unknown]").unwrap();
                continue;
            }

            for (symbol_idx, symbol) in symbols.iter().enumerate() {
                let indent = if symbol_idx == 0 { "      " } else { "        ↳ " };
                let name = symbol.name().unwrap_or("[unknown]");

                write!(output, "{indent}at {name}").unwrap();

                match (symbol.file(), symbol.line()) {
                    (Some(file), Some(line)) => {
                        let file_str = file.to_string_lossy();
                        // Truncate long file paths
                        let display_file = if file_str.len() > 50 {
                            format!("...{}", &file_str[file_str.len() - 47..])
                        } else {
                            file_str.to_string()
                        };
                        writeln!(output, " ({display_file}:{line})").unwrap();
                    },
                    (Some(file), None) => {
                        let file_str = file.to_string_lossy();
                        writeln!(output, " ({file_str})").unwrap();
                    },
                    (None, Some(line)) => {
                        writeln!(output, " (line {line})").unwrap();
                    },
                    (None, None) => {
                        writeln!(output).unwrap();
                    }
                }
            }
        }
    }
    output
}
}
}
