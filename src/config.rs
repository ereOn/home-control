use std::{net::SocketAddr, path::PathBuf};

use clap::{App, Arg};

const ARG_STATIC_ROOT: &str = "static-root";
const ARG_LISTEN_ENDPOINT: &str = "listen-endpoint";
const ARG_REVERSE_PROXY_URL: &str = "reverse-proxy-url";

const DEFAULT_STATIC_ROOT: &str = "static";
const DEFAULT_LISTEN_ENDPOINT: &str = "127.0.0.1:8000";

pub struct Config {
    pub static_root: PathBuf,
    pub listen_endpoint: SocketAddr,
    pub reverse_proxy_url: Option<String>,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let matches = App::new("home-control")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::with_name(ARG_STATIC_ROOT)
                    .long(ARG_STATIC_ROOT)
                    .value_name("root")
                    .help(&format!(
                        "The root to the static files to serve. Defaults to '{}'",
                        DEFAULT_STATIC_ROOT,
                    ))
                    .takes_value(true),
            )
            .arg(
                Arg::with_name(ARG_LISTEN_ENDPOINT)
                    .long(ARG_LISTEN_ENDPOINT)
                    .short("l")
                    .value_name("endpoint")
                    .help(&format!(
                        "The endpoint to listen on. Defaults to '{}'",
                        DEFAULT_LISTEN_ENDPOINT,
                    ))
                    .takes_value(true),
            )
            .arg(
                Arg::with_name(ARG_REVERSE_PROXY_URL)
                    .long(ARG_REVERSE_PROXY_URL)
                    .short("r")
                    .value_name("url")
                    .help("The URL to another server to proxy requests to. Default: does not use a proxy")
                    .takes_value(true),
            )
            .get_matches();

        Ok(Self {
            static_root: matches
                .value_of(ARG_STATIC_ROOT)
                .unwrap_or(DEFAULT_STATIC_ROOT)
                .parse()
                .unwrap(),
            listen_endpoint: matches
                .value_of(ARG_LISTEN_ENDPOINT)
                .unwrap_or(DEFAULT_LISTEN_ENDPOINT)
                .parse()?,
            reverse_proxy_url: matches
                .value_of(ARG_REVERSE_PROXY_URL)
                .map(|s| s.to_string()),
        })
    }
}
