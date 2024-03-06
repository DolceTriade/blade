use actix_web::*;
use anyhow::Context;
use futures::prelude::future::FutureExt;
use std::net::SocketAddr;
use tracing::instrument;

#[instrument]
pub async fn run_admin_server(
    admin_host: SocketAddr,
    filter_channel: tokio::sync::mpsc::Sender<String>,
    span_channel: tokio::sync::mpsc::Sender<bool>,
) -> anyhow::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(filter_channel.clone()))
            .app_data(web::Data::new(span_channel.clone()))
            .service(set_filter)
            .service(set_span)
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
