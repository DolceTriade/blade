use cfg_if::cfg_if;
pub mod routes;
pub mod components;

// boilerplate to run in different modes
cfg_if! {
    // server-only stuff
    if #[cfg(feature = "ssr")] {
        use leptos::*;
        use actix_files::{Files};
        use actix_web::*;
        use leptos_actix::{generate_route_list, LeptosRoutes};
        use pretty_env_logger;
        use crate::routes::app::App;

        #[actix_web::main]
        async fn main() -> std::io::Result<()> {
            pretty_env_logger::init();
            // Setting this to None means we'll be using cargo-leptos and its env vars.
            // when not using cargo-leptos None must be replaced with Some("Cargo.toml")
            let conf = get_configuration(Some("blade/leptos.toml")).await.unwrap();

            let addr = conf.leptos_options.site_addr;
            let routes = generate_route_list(App);
            log::info!("Starting blade server at: {}", addr.to_string());
            HttpServer::new(move || {
                let leptos_options = &conf.leptos_options;
                let site_root = &leptos_options.site_root;
                App::new()
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
                        App,
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
