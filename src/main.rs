use log::info;

use home_control::{api::Api, home_assistant::Client};
use rust_embed::RustEmbed;
use warp::Filter;
use warp_reverse_proxy::reverse_proxy_filter;

#[derive(RustEmbed)]
#[folder = "frontend/build"]
struct Data;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = home_control::config::Config::new()?;
    home_control::log::init(config.debug);

    info!("Home-control, version {}", env!("CARGO_PKG_VERSION"));

    let ha_client =
        Client::new(&config.home_assistant_endpoint, config.home_assistant_token).await?;
    let ha_controller = ha_client.new_controller();

    let api = Api::new(config.gpio_config, ha_controller)?;
    let routes = api.routes();

    if let Some(reverse_proxy_url) = config.reverse_proxy_url {
        info!(
            "Serving files from reverse proxy at `{}`",
            reverse_proxy_url
        );

        tokio::select! {
            r = ha_client.run() => r?,
            _ = warp::serve(routes.or(reverse_proxy_filter("".to_string(), reverse_proxy_url)))
                .run(config.listen_endpoint) => {},
        }
    } else {
        info!("Serving static files.",);

        tokio::select! {
            r = ha_client.run() => r?,
            _ = warp::serve(routes.or(warp_embed::embed(&Data)))
                .run(config.listen_endpoint) => {},
        }
    };

    Ok(())
}
