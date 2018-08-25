use std::path::PathBuf;

use actix::prelude::*;
use actix_web::{self, middleware, App};

use client;
use lobby::Lobby;

const ROOT_PATH: &str = "/";
const WEBSOCKET_PATH: &str = "/ws";

/// Application state
pub struct AppState {
    pub lobby: Addr<Lobby>,
}

/// Launch the Actix-web server.
pub fn do_server(bind: &str, static_path: &str) {
    let sys = actix::System::new("webrtc-demo-server");
    let static_path: PathBuf = PathBuf::from(static_path);

    // Confirm that the static files are present
    let mut index_page = static_path.clone();
    index_page.push("index.html");
    if !index_page.exists() {
        println!("Error: Index page not found: {}", index_page.to_string_lossy());
        println!("Are static files present?  Do you need to use \"--static-path=...\"?");
        ::std::process::exit(1);
    }

    // Start lobby actor
    let lobby = Arbiter::start(|_| Lobby::new());

    // Start http server
    actix_web::server::new(move || {
        let state = AppState {
            lobby: lobby.clone()
        };

        App::with_state(state)
            .resource(WEBSOCKET_PATH, |r| r.route().f(client::ws_index))
            .handler(ROOT_PATH,
                    actix_web::fs::StaticFiles::new(&static_path).unwrap()
                        .index_file("index.html")
                        .show_files_listing())
            // logger
            .middleware(middleware::Logger::default())
    }).bind(bind)
    .unwrap()
    .start();

    println!("Started http server: {}", bind);
    let _ = sys.run();
}
