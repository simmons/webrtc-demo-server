//! WebRTC Demo Server

extern crate actix;
extern crate actix_web;
extern crate clap;
extern crate futures;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate rand;
extern crate env_logger;
extern crate woothee;

use clap::{App as Clap, Arg};

mod client;
mod lobby;
mod names;
mod server;

const DEFAULT_BIND: &str = "127.0.0.1:8080";
const DEFAULT_STATIC_PATH: &str = "static/";

fn main() {
    // Parse command-line arguments and dispatch
    let app = Clap::new("WebRTC Demo Server")
        .version("0.1.0")
        .about("Demonstrate WebRTC with a backend signalling server.")
        .arg(
            Arg::with_name("bind")
                .short("b")
                .long("bind")
                .takes_value(true)
                .default_value(DEFAULT_BIND)
                .help("Specify the ip:port for binding.")
                .required(false),
        ).arg(
            Arg::with_name("static_path")
                .short("s")
                .long("static-path")
                .takes_value(true)
                .default_value(DEFAULT_STATIC_PATH)
                .help("Path to static resources.")
                .required(false),
        );
    let matches = app.get_matches();

    if let Err(_) = ::std::env::var("RUST_LOG") {
        ::std::env::set_var(
            "RUST_LOG",
            "actix=info,actix_web=info,webrtc_demo_server=trace",
        );
    }
    env_logger::init();

    // Launch the web server
    server::do_server(
        matches.value_of("bind").unwrap(),
        matches.value_of("static_path").unwrap(),
    );
}
