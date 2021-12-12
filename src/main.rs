use log::info;

use rust_embed::RustEmbed;
use warp::Filter;
use warp_reverse_proxy::reverse_proxy_filter;

#[derive(RustEmbed)]
#[folder = "frontend/build"]
struct Data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    home_control::log::init();
    let config = home_control::config::Config::new()?;

    info!("Home-control, version {}", env!("CARGO_PKG_VERSION"));

    let api = home_control::api::Api::new(config.api_config)?;
    let routes = api.routes();

    if let Some(reverse_proxy_url) = config.reverse_proxy_url {
        info!(
            "Serving files from reverse proxy at `{}`",
            reverse_proxy_url
        );

        warp::serve(routes.or(reverse_proxy_filter("".to_string(), reverse_proxy_url)))
            .run(config.listen_endpoint)
            .await
    } else {
        info!("Serving static files.",);

        warp::serve(routes.or(warp_embed::embed(&Data)))
            .run(config.listen_endpoint)
            .await
    }

    Ok(())
}
