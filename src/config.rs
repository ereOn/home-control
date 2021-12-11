use std::net::SocketAddr;

use clap::{App, Arg};

const ARG_LISTEN_ENDPOINT: &str = "listen-endpoint";
const DEFAULT_LISTEN_ENDPOINT: &str = "127.0.0.1:8000";

pub struct Config {
    pub listen_endpoint: SocketAddr,
}

impl Config {
    pub fn new() -> Self {
        let matches = App::new("home-control")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::with_name(ARG_LISTEN_ENDPOINT)
                    .short("l")
                    .long(ARG_LISTEN_ENDPOINT)
                    .value_name("endpoint")
                    .help(&format!(
                        "The endpoint to listen on. Default: {}",
                        DEFAULT_LISTEN_ENDPOINT,
                    ))
                    .takes_value(true),
            )
            .get_matches();

        Self {
            listen_endpoint: matches
                .value_of(ARG_LISTEN_ENDPOINT)
                .unwrap_or(DEFAULT_LISTEN_ENDPOINT)
                .parse()
                .unwrap(),
        }
    }
}
