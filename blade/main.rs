use cfg_if::cfg_if;
mod counters;

// boilerplate to run in different modes
cfg_if! {
    // server-only stuff
    if #[cfg(feature = "ssr")] {
        use leptos::*;
        use actix_files::{Files};
        use actix_web::*;
        use crate::counters::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use pretty_env_logger;

        #[get("/api/events")]
        async fn counter_events() -> impl Responder {
            use futures::StreamExt;

            let stream =
                futures::stream::once(async { crate::counters::get_server_count().await.unwrap_or(0) })
                    .chain(COUNT_CHANNEL.clone())
                    .map(|value| {
                        Ok(web::Bytes::from(format!(
                            "event: message\ndata: {value}\n\n"
                        ))) as Result<web::Bytes>
                    });
            HttpResponse::Ok()
                .insert_header(("Content-Type", "text/event-stream"))
                .streaming(stream)
        }

        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            pretty_env_logger::init();
            // Setting this to None means we'll be using cargo-leptos and its env vars.
            // when not using cargo-leptos None must be replaced with Some("Cargo.toml")
            let conf = get_configuration(Some("blade/leptos.toml")).await.unwrap();

            let addr = conf.leptos_options.site_addr;
            let routes = generate_route_list(|cx| view! { cx, <Counters/> });
            log::info!("Starting blade server at: {}", addr.to_string());
            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;
                App::new()
                    .service(counter_events)
                    .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
                    // serve JS/WASM/CSS from `pkg`
                    .service(Files::new("/pkg", site_root))
                    // serve other assets from the `assets` directory
                    .service(Files::new("/assets", format!("{site_root}/static")))
                    // serve the favicon from /favicon.ico
                    .service(favicon)
                    .leptos_routes(
                        leptos_options.to_owned(),
                        routes.to_owned(),
                        Counters,
                    )
                    .app_data(web::Data::new(leptos_options.to_owned()))
                    .wrap(middleware::Logger::new("%t -- %a %s %U"))
            })
            .bind(&addr)?
            .run()
            .await
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
