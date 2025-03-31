use actix_web::*;
use anyhow::Context;
use futures::prelude::future::FutureExt;
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
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

#[derive(Debug,serde::Deserialize)]
struct MemdumpEnableQuery {
    enable: bool,
}

#[get("/admin/mem/enable")]
#[instrument]
async fn debug_mem_profile_enable_handler(info: web::Query<MemdumpEnableQuery>) -> Result<HttpResponse> {
    memdump::enable_profiling(info.enable).await.map_err(|e| {
        error::ErrorInternalServerError(format!("error setting memdump status: {e:#?}"))
    })?;
    Ok(HttpResponse::Ok().into())
}
