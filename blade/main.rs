use std::net::SocketAddr;

use cfg_if::cfg_if;
use std::io::IsTerminal;
use tracing::{event, instrument, level_filters::LevelFilter};
use tracing_actix_web::{DefaultRootSpanBuilder, RootSpanBuilder};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, reload::Handle, EnvFilter, Layer,
};
pub mod components;
pub mod routes;

// boilerplate to run in different modes
cfg_if! {
    // server-only stuff
    if #[cfg(feature = "ssr")] {
        use anyhow::Context;
        use actix_files::Files;
        use actix_web::*;
        use tracing_actix_web::TracingLogger;
        use clap::*;
        use futures::join;
        use leptos::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use std::sync::Arc;
        use runfiles::Runfiles;

        pub mod admin;

        use crate::routes::app::App;

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
        }

        fn fmt_layer<S>(show_spans: bool) -> Box<dyn Layer<S> + Sync + Send>
        where S: for<'a> tracing_subscriber::registry::LookupSpan<'a> + tracing::Subscriber{
            let use_ansi = std::io::stdout().is_terminal();
            let mut fmt_layer = tracing_subscriber::fmt::layer()
                .with_ansi(use_ansi)
                .compact()
                .with_file(true)
                .with_line_number(true);

            if show_spans {
                fmt_layer = fmt_layer.with_span_events(FmtSpan::CLOSE);
            }

            fmt_layer.boxed()
        }

        type SpanHandle = Handle<Box<dyn Layer<Registry> + Send + Sync>, Registry>;
        fn init_logging() -> (Handle<EnvFilter, impl Sized>, SpanHandle) {
            let env_filter = tracing_subscriber::EnvFilter::builder().with_default_directive(LevelFilter::INFO.into()).from_env_lossy();
            let fmt_layer = fmt_layer(false);
            let (layer, span_handle) = tracing_subscriber::reload::Layer::new(fmt_layer);
            let (filter, handle) = tracing_subscriber::reload::Layer::new(env_filter);

            tracing_subscriber::registry().with(layer).with(filter).init();

            (handle, span_handle)
        }

        #[actix_web::main]
        async fn main() -> anyhow::Result<()> {
            // install global subscriber configured based on RUST_LOG envvar.
            let (filter_handle, span_handle) = init_logging();

            let args = Args::parse();

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
                            if let Some(e) = span_handle.reload(fmt_layer(enable)).err() {
                                tracing::error!("error setting span enable: {e}");
                            }
                        },
                        None => {
                            tracing::error!("None filter received by set_span");
                            break;
                        },
                    }
                }
            });

            // Setting this to None means we'll be using cargo-leptos and its env vars.
            // when not using cargo-leptos None must be replaced with Some("Cargo.toml")
            let r = Runfiles::create().expect("Must run using bazel with runfiles");
            let leptos_toml = r.rlocation("_main/blade/leptos.toml");
            let assets = r.rlocation("_main/blade/static");
            let pkg = r.rlocation("_main/blade");
            let mut conf = get_configuration(Some(leptos_toml.to_str().unwrap())).await.unwrap();
            conf.leptos_options.site_addr = args.http_host;
            let addr = conf.leptos_options.site_addr;
            let routes = generate_route_list(App);
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
                let leptos_options = &conf.leptos_options;
                let fn_state = actix_state.clone();
                let rt_state = actix_state.clone();
                App::new()
                    .route("/api/{tail:.*}", leptos_actix::handle_server_fns_with_context(move|| provide_context(fn_state.clone())))
                    // serve JS/WASM/CSS from `pkg`
                    .service(Files::new("/pkg", pkg.clone()))
                    // serve other assets from the `assets` directory
                    .service(Files::new("/assets", assets.clone()))
                    // serve the favicon from /favicon.ico
                    .service(favicon)
                    .leptos_routes_with_context(
                        leptos_options.to_owned(),
                        routes.to_owned(),
                        move|| provide_context(rt_state.clone()),
                        App,
                    )
                    .app_data(web::Data::new(leptos_options.to_owned()))
                    .wrap(TracingLogger::<BladeRootSpanBuilder>::new())
            })
            .disable_signals()
            .bind(&addr)?
            .run();
            let fut2 = bep::run_bes_grpc(args.grpc_host, state, &args.debug_message_pattern);
            let fut3 = periodic_cleanup(cleanup_state);
            let fut4 = admin::run_admin_server(args.admin_host, filter_tx, span_tx);

            let res = join!(fut1, fut2, fut3, fut4, set_filter_fut, set_span_fut);
            if res.0.is_ok() && res.1.is_ok() {
                return Ok(());
            }
            if res.0.is_err() {
                return res.0.context("server failed");
            } else {
                return res.1.context("grpc failed");
            }
        }

        #[actix_web::get("favicon.ico")]
        #[instrument]
        async fn favicon() -> actix_web::Result<actix_files::NamedFile> {
            let r = Runfiles::create().expect("Must run using bazel with runfiles");
            Ok(actix_files::NamedFile::open(r.rlocation("_main/blade/static/favicon.ico"))?)
        }

        #[instrument]
        async fn periodic_cleanup(global: Arc<state::Global>) {
            let Some(interval) = global.retention else { return; };
            let day = std::time::Duration::from_secs(60 * 60 * 24);
            let check_interval = std::cmp::min(day, interval/7);
            loop {
                tokio::time::sleep(check_interval).await;
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

            fn on_request_end<B: body::MessageBody>(span: tracing::Span, outcome: &std::prelude::v1::Result<dev::ServiceResponse<B>, actix_web::error::Error>) {
                if let Ok(response) = &outcome {
                    if response.response().error().is_none() {
                        event!(tracing::Level::DEBUG, "OK");
                    }
                }
                DefaultRootSpanBuilder::on_request_end(span, outcome)
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
