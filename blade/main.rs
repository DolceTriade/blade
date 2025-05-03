use std::fmt::Write;
use std::net::SocketAddr;
use std::sync::Mutex;

use actix_web::body::MessageBody;
use actix_web::dev::ServiceResponse;
use actix_web::http::{Method, StatusCode};
use cfg_if::cfg_if;
use prometheus_client::encoding::{EncodeLabelSet, EncodeLabelValue};
use std::io::IsTerminal;
use tracing::Span;
use tracing::{instrument, level_filters::LevelFilter};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, reload::Handle, EnvFilter, Layer,
};
#[allow(clippy::empty_docs)]
pub mod components;
#[allow(clippy::empty_docs)]
pub mod routes;

// boilerplate to run in different modes
cfg_if! {
    // server-only stuff
    if #[cfg(feature = "ssr")] {
        use actix_files::Files;
        use actix_web::*;
        use tracing_actix_web::TracingLogger;
        use clap::*;
        use leptos::prelude::*;
        use leptos_meta::MetaTags;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use std::sync::Arc;
        use runfiles::Runfiles;
        use lazy_static::lazy_static;
        use prometheus_client::metrics::family::Family;
        use prometheus_client::metrics::counter::Counter;
        use tikv_jemallocator::Jemalloc;

        #[global_allocator]
        static GLOBAL: Jemalloc = Jemalloc;


        pub mod admin;

        use crate::routes::app::App;

        lazy_static! {
            static ref API_ERRORS: Family::<APIErrorLabels, Counter> = metrics::register_metric("blade_http_errors", "Actix API requests errors", Family::default());
            static ref API_REQUESTS: Family::<APIRequestLabels, Counter> = metrics::register_metric("blade_http_requests", "Actix API requests", Family::default());
        }

        #[derive(Clone, Debug, Hash, PartialEq, Eq)]
        struct HTTPMethod(Method);

        impl EncodeLabelValue for HTTPMethod {
            fn encode(&self, encoder: &mut prometheus_client::encoding::LabelValueEncoder) -> std::prelude::v1::Result<(), std::fmt::Error> {
                encoder.write_str(self.0.as_str())
            }
        }

        impl From<Method> for HTTPMethod {
            fn from(value: Method) -> Self {
                Self(value)
            }
        }

        #[derive(Clone, Debug, Hash, PartialEq, Eq)]
        struct HTTPStatusCode(StatusCode);

        impl EncodeLabelValue for HTTPStatusCode {
            fn encode(&self, encoder: &mut prometheus_client::encoding::LabelValueEncoder) -> std::prelude::v1::Result<(), std::fmt::Error> {
                encoder.write_str(self.0.as_str())
            }
        }

        impl From<StatusCode> for HTTPStatusCode {
            fn from(value: StatusCode) -> Self {
                Self(value)
            }
        }

        #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
        struct APIRequestLabels {
            method: HTTPMethod,
            path: String
        }

        #[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
        struct APIErrorLabels {
            method: HTTPMethod,
            path: String,
            code: HTTPStatusCode
        }


        #[derive(Parser)]
        #[command(name = "Blade")]
        #[command(about = "Bazel Build Event Service app")]
        struct Args {
            #[arg(short='g', long="grpc_host", value_name = "GRPC_HOST", default_value="[::]:50332")]
            grpc_host: SocketAddr,
            #[arg(short='H', long="http_host", value_name = "HTTP_HOST", default_value="[::]:3000")]
            http_host: SocketAddr,
            #[arg(short='A', long="admin_host", value_name = "ADMIN_HOST", default_value="[::]:3001")]
            admin_host: SocketAddr,
            #[arg(short='p', long="db_path", value_name = "DATABASE_PATH", default_value="")]
            db_path: String,
            #[arg(short='d', long="print_message", value_name = "PATTERN", default_value="")]
            debug_message_pattern: String,
            #[arg(short='l', long="allow_local", value_name = "ALLOW_LOCAL", default_value="false")]
            allow_local: bool,
            #[arg(short='o', long="bytestream_override", value_name = "OVERRIDE")]
            bytestream_overrides: Vec<String>,
            #[arg(short='r', long="retention", value_name = "RETENTION", value_parser = humantime::parse_duration)]
            retention: Option<std::time::Duration>,
            #[arg(short='s', long="session_lock_time", value_name = "LOCK_TIME", value_parser = humantime::parse_duration, default_value="5m")]
            session_lock_time: std::time::Duration,
            #[arg(long="flame", value_name = "FLAME")]
            flame_path: Option<String>,
            #[arg(long="json", value_name="JSON", default_value="false")]
            json: bool,
        }

        fn fmt_layer<S>(show_spans: bool, json: bool) -> Box<dyn Layer<S> + Sync + Send>
        where S: for<'a> tracing_subscriber::registry::LookupSpan<'a> + tracing::Subscriber{
            let use_ansi = std::io::stdout().is_terminal();
            let mut fmt_layer = tracing_subscriber::fmt::layer()
                .with_ansi(use_ansi)
                .with_file(true)
                .with_line_number(true);

            if show_spans {
                fmt_layer = fmt_layer.with_span_events(FmtSpan::CLOSE);
            }

            if json {
                return fmt_layer.json().boxed();
            }

            fmt_layer.compact().boxed()
        }

        type SpanHandle = Handle<Box<dyn Layer<Registry> + Send + Sync>, Registry>;
        fn init_logging(flame: Option<String>, json: bool) -> (Handle<EnvFilter, impl Sized>, SpanHandle, Option<impl Drop>) {
            let env_filter = tracing_subscriber::EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env_lossy();
            let fmt_layer = fmt_layer(false, json);
            let (layer, span_handle) = tracing_subscriber::reload::Layer::new(fmt_layer);
            let (filter, handle) = tracing_subscriber::reload::Layer::new(env_filter);


            let reg = tracing_subscriber::registry().with(layer).with(filter);

            let mut guard_opt = None;
            if let Some(flame_path) = flame {
                let (flame, guard) = tracing_flame::FlameLayer::with_file(flame_path).expect("error creating flame logger");
                reg.with(flame).init();
                guard_opt = Some(guard);
            } else {
                reg.init();
            }


            (handle, span_handle, guard_opt)
        }

        #[actix_web::main]
        async fn main() -> anyhow::Result<()> {
            let args = Args::parse();
            // install global subscriber configured based on RUST_LOG envvar.
            let (filter_handle, span_handle, _guard) = init_logging(args.flame_path.clone(), args.json);


            let (filter_tx, mut filter_rx) = tokio::sync::mpsc::channel::<String>(3);
            let set_filter_fut = tokio::spawn(async move {
                loop {
                    match filter_rx.recv().await {
                        Some(filter) => {
                            let span = tracing::span!(tracing::Level::INFO, "set_filter", filter=filter);
                            let _e = span.enter();
                            tracing::info!("Setting log filter: {filter}");
                            match tracing_subscriber::filter::EnvFilter::builder().parse(&filter) {
                                Ok(f) => {
                                    if let Some(e) = filter_handle.reload(f).err() {
                                        tracing::error!("error setting filter: {e}");
                                    }
                                },
                                Err(e) => { tracing::error!("error parsing filter: {e}"); },
                            }
                        },
                        None => {
                            tracing::error!("None filter received by set_filter");
                            break;
                        },
                    }
                }
            });
            let (span_tx, mut span_rx) = tokio::sync::mpsc::channel::<bool>(3);
            let set_span_fut = tokio::spawn(async move {
                loop {
                    match span_rx.recv().await {
                        Some(enable) => {
                            let span = tracing::span!(tracing::Level::INFO, "set_span", enable=enable);
                            let _e = span.enter();
                            tracing::info!("Setting span enable: {enable}");
                            if let Some(e) = span_handle.reload(fmt_layer(enable, args.json)).err() {
                                tracing::error!("error setting span enable: {e}");
                            }
                        },
                        None => {
                            tracing::error!("`None` filter received by set_span");
                            break;
                        },
                    }
                }
            });
            let cur = std::env::current_dir().unwrap();
            let ld_preload = std::env::var("LD_PRELOAD").unwrap_or("".to_string());
            tracing::info!("LD_PRELOAD is {ld_preload}; pwd is {cur:?}");

            // Setting this to None means we'll be using cargo-leptos and its env vars.
            // when not using cargo-leptos None must be replaced with Some("Cargo.toml")
            let r = Runfiles::create().expect("Must run using bazel with runfiles");
            let leptos_toml = r.rlocation("blade/blade/leptos.toml").unwrap();
            let assets = r.rlocation("blade/blade/static/static").unwrap();
            let mut conf = get_configuration(Some(leptos_toml.to_str().unwrap())).unwrap();
            conf.leptos_options.site_addr = args.http_host;
            let addr = conf.leptos_options.site_addr;
            let mut bs = bytestream::Client::new();
            for o in &args.bytestream_overrides {
                if let Some(s) = o.split_once('=') {
                    bs.add_override(s.0, s.1);
                }
            }
            let db_manager = db::new(&args.db_path)?;
            let state = Arc::new(state::Global { db_manager, allow_local: args.allow_local, bytestream_client: bs, retention: args.retention, session_lock_time: args.session_lock_time });
            let actix_state = state.clone();
            let cleanup_state = state.clone();
            tracing::info!("Starting blade server at: {}", addr.to_string());
            let fut1 = HttpServer::new(move || {
                let leptos_options = conf.leptos_options.clone();
                let rt_state = actix_state.clone();
                let routes = generate_route_list(App);
                App::new()
                    // serve JS/WASM/CSS and other assets from `/assets`
                    .service(Files::new("/assets", assets.clone()))
                    // serve the favicon from /favicon.ico
                    .service(favicon)
                    .leptos_routes_with_context(
                        routes,
                        move|| provide_context(rt_state.clone()),
                        move|| {
                            view! {
                            <!DOCTYPE html>
                            <html lang="en">
                                <head>
                                    <meta charset="utf-8"/>
                                    <meta name="viewport" content="width=device-width, initial-scale=1"/>
                                    <AutoReload options=leptos_options.to_owned() />
                                    <HydrationScripts options=leptos_options.to_owned()/>
                                    <MetaTags/>
                                </head>
                                <body>
                                    <App/>
                                </body>
                            </html>
                        }})
                    .app_data(web::Data::new(conf.leptos_options.to_owned()))
                    .wrap(TracingLogger::<BladeRootSpanBuilder>::new())
            })
            .bind(&addr)?
            .run();
            let re_handle = Arc::new(Mutex::new(regex::Regex::new(&args.debug_message_pattern)?));
            let fut2 = bep::run_bes_grpc(args.grpc_host, state, re_handle.clone());
            let fut3 = periodic_cleanup(cleanup_state);
            let fut4 = admin::run_admin_server(args.admin_host, filter_tx, span_tx, re_handle);

            tokio::select! {
                e = fut1 => { tracing::info!("Main HTTP server finished: {e:#?}"); },
                e = fut2 => { tracing::info!("gRPC server finished: {e:#?}"); },
                e = fut3 => { tracing::info!("periodic cleanup finished: {e:#?}"); },
                e = fut4 => { tracing::info!("Admin HTTP server finished: {e:#?}"); },
                _ = set_filter_fut => {},
                _ = set_span_fut => {},
            }

            Ok(())
        }

        #[actix_web::get("favicon.ico")]
        #[instrument]
        async fn favicon() -> actix_web::Result<actix_files::NamedFile> {
            let r = Runfiles::create().expect("Must run using bazel with runfiles");
            Ok(actix_files::NamedFile::open(r.rlocation("blade/blade/static/static/favicon.ico").unwrap())?)
        }

        #[instrument]
        async fn periodic_cleanup(global: Arc<state::Global>) {
            let day = std::time::Duration::from_secs(60 * 60 * 24);
            let interval = global.retention.unwrap_or(day);
            let check_interval = std::cmp::min(day, interval/7);
            loop {
                tokio::time::sleep(check_interval).await;
                if global.retention.is_none() {
                    continue;
                }
                let Ok(mut db) = global.db_manager.get() else {
                    tracing::warn!("Failed to get DB handle for cleanup");
                    continue;
                };
                let Some(since) = std::time::SystemTime::now().checked_sub(interval) else {
                    tracing::warn!("Overflow when clean up time");
                    continue;
                };
                tracing::info!("Cleanup result: {:#?}", db.delete_invocations_since(&since));
            }
        }

        #[derive(Default)]
        pub(crate) struct BladeRootSpanBuilder;

        impl RootSpanBuilder for BladeRootSpanBuilder {
            fn on_request_start(request: &dev::ServiceRequest) -> tracing::Span {
                DefaultRootSpanBuilder::on_request_start(request)
            }

            fn on_request_end<B: MessageBody>(span: Span, outcome: &core::result::Result<ServiceResponse<B>, actix_web::error::Error>) {
                match &outcome {
                    Ok(response) => {
                        let method = response.request().method().clone();
                        let path = response.request().match_pattern().unwrap_or("unknown".to_string());
                        API_REQUESTS.get_or_create(&APIRequestLabels{ method: method.clone().into(), path: path.clone() }).inc();
                        if let Some(error) = response.response().error() {
                            // use the status code already constructed for the outgoing HTTP response
                            API_ERRORS.get_or_create(&APIErrorLabels{ method: method.into(), code: response.status().into(), path: path.clone()}).inc();
                            handle_error(span, response.status(), error.as_response_error());
                        } else {
                            if !response.response().status().is_success() {
                                API_ERRORS.get_or_create(&APIErrorLabels{ method: method.into(), code: response.status().into(), path: path.clone()}).inc();
                            }
                            let code: i32 = response.response().status().as_u16().into();
                            span.record("http.status_code", code);
                            span.record("otel.status_code", "OK");
                        }
                    }
                    Err(error) => {
                        let method = Method::TRACE;
                        let path = "unknown";
                        API_REQUESTS.get_or_create(&APIRequestLabels{ method: method.clone().into(), path: path.to_string() }).inc();
                        let response_error = error.as_response_error();
                        API_ERRORS.get_or_create(&APIErrorLabels{ method: method.into(), code: response_error.status_code().into(), path: path.to_string()}).inc();
                        handle_error(span, response_error.status_code(), response_error);
                    }
                };
            }
        }

        fn handle_error(span: Span, status_code: StatusCode, response_error: &dyn ResponseError) {
            // pre-formatting errors is a workaround for https://github.com/tokio-rs/tracing/issues/1565
            let display = format!("{response_error}");
            let debug = format!("{response_error:?}");
            span.record("exception.message", tracing::field::display(display));
            span.record("exception.details", tracing::field::display(debug));
            let code: i32 = status_code.as_u16().into();

            span.record("http.status_code", code);

            if status_code.is_client_error() {
                span.record("otel.status_code", "OK");
            } else {
                span.record("otel.status_code", "ERROR");
            }
        }
    }
    // client-only main for Trunk
    else {
        pub fn main() {
            // isomorphic counters cannot work in a Client-Side-Rendered only
            // app as a server is required to maintain state
        }
    }
}
