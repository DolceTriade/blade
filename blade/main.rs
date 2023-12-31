use std::net::SocketAddr;

use cfg_if::cfg_if;
pub mod components;
pub mod routes;

// boilerplate to run in different modes
cfg_if! {
    // server-only stuff
    if #[cfg(feature = "ssr")] {
        use anyhow::Context;
        use actix_files::Files;
        use actix_web::*;
        use clap::*;
        use futures::join;
        use leptos::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use std::sync::Arc;
        use runfiles::Runfiles;

        use crate::routes::app::App;

        #[derive(Parser)]
        #[command(name = "Blade")]
        #[command(about = "Bazel Build Event Service app")]
        struct Args {
            #[arg(short='g', long="grpc_host", value_name = "GRPC_HOST", default_value="[::]:50332")]
            grpc_host: SocketAddr,
            #[arg(short='H', long="http_host", value_name = "HTTP_HOST", default_value="[::]:3000")]
            http_host: SocketAddr,
            #[arg(short='p', long="db_path", value_name = "DATABASE_PATH", default_value="")]
            db_path: String,
            #[arg(short='d', long="print_message", value_name = "PATTERN", default_value="")]
            debug_message_pattern: String,
            #[arg(short='l', long="allow_local", value_name = "ALLOW_LOCAL", default_value="false")]
            allow_local: bool,
            #[arg(short='o', long="bytestream_override", value_name = "OVERRIDE")]
            bytestream_overrides: Vec<String>,

        }

        #[actix_web::main]
        async fn main() -> anyhow::Result<()> {
            if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", "info");
            }
            pretty_env_logger::init();

            let args = Args::parse();

            // Setting this to None means we'll be using cargo-leptos and its env vars.
            // when not using cargo-leptos None must be replaced with Some("Cargo.toml")
            let r = Runfiles::create().expect("Must run using bazel with runfiles");
            let leptos_toml = r.rlocation("blade/blade/leptos.toml");
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
            let state = Arc::new(state::Global { db_manager, allow_local: args.allow_local, bytestream_client: bs });
            let actix_state = state.clone();
            log::info!("Starting blade server at: {}", addr.to_string());
            let fut1 = HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;
                let fn_state = actix_state.clone();
                let rt_state = actix_state.clone();
                App::new()
                    .route("/api/{tail:.*}", leptos_actix::handle_server_fns_with_context(move|| provide_context(fn_state.clone())))
                    // serve JS/WASM/CSS from `pkg`
                    .service(Files::new("/pkg", site_root))
                    // serve other assets from the `assets` directory
                    .service(Files::new("/assets", format!("{site_root}/static")))
                    // serve the favicon from /favicon.ico
                    .service(favicon)
                    .leptos_routes_with_context(
                        leptos_options.to_owned(),
                        routes.to_owned(),
                        move|| provide_context(rt_state.clone()),
                        App,
                    )
                    .app_data(web::Data::new(leptos_options.to_owned()))
                    .wrap(middleware::Logger::new("%t -- %a %s %U")
                        .exclude("/")
                        .exclude("/favicon.ico")
                        .exclude_regex("/pkg/.*"))
            })
            .disable_signals()
            .bind(&addr)?
            .run();
            let fut2 = bep::run_bes_grpc(args.grpc_host, state, &args.debug_message_pattern);
            let res = join!(fut1, fut2);
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
        async fn favicon(
            leptos_options: actix_web::web::Data<leptos::LeptosOptions>,
        ) -> actix_web::Result<actix_files::NamedFile> {
            let leptos_options = leptos_options.into_inner();
            let site_root = &leptos_options.site_root;
            Ok(actix_files::NamedFile::open(format!(
                "{site_root}/static/favicon.ico"
            ))?)
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
