use log::info;

#[tokio::main]
async fn main() {
    home_control::log::init();
    let config = home_control::config::Config::new();

    info!("Home-control, version {}", env!("CARGO_PKG_VERSION"));

    let routes = warp::fs::dir("static");

    warp::serve(routes).run(config.listen_endpoint).await
}
