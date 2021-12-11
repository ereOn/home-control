use actix_web::{
    client::Client, post, web::ServiceConfig, App, HttpResponse, HttpServer, Responder,
};
use actix_web_static_files;
use log::info;
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

fn configure_app(cfg: &mut ServiceConfig) {
    cfg.data(Client::new());

    cfg.service(actix_web_static_files::ResourceFiles::new("/", generate()))
        .service(echo);
}

const ARG_LISTEN_ENDPOINT: &str = "listen-endpoint";
const DEFAULT_LISTEN_ENDPOINT: &str = "127.0.0.1:8000";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let matches = clap::App::new("home-control")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            clap::Arg::with_name(ARG_LISTEN_ENDPOINT)
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

    let listen_endpoint = matches
        .value_of(ARG_LISTEN_ENDPOINT)
        .unwrap_or("127.0.0.1:8000");

    info!("Home-control, version {}", env!("CARGO_PKG_VERSION"));

    HttpServer::new(|| App::new().configure(configure_app))
        .bind(listen_endpoint)?
        .run()
        .await
}
