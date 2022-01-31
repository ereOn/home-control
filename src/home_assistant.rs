use anyhow::{Context, Result};
use futures_util::{Sink, Stream, StreamExt};
use log::info;
use serde::Deserialize;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as WsError, Message as WsMessage},
};
use url::Url;

trait WebSocket<Item = WsMessage, Error = WsError>:
    Sink<Item, Error = Error> + Stream<Item = Result<Item, Error>> + Unpin
{
}

impl<T> WebSocket for T where
    T: Sink<WsMessage, Error = WsError> + Stream<Item = Result<WsMessage, WsError>> + Unpin
{
}

pub struct Client {
    ws: Box<dyn WebSocket>,
}

impl Client {
    pub async fn new(endpoint: &String, token: &String) -> Result<Self> {
        let url = Url::parse(&format!("wss://{}/api/websocket", endpoint))
            .context("failed to parse home-assistant endpoint")?;

        info!("Connecting to Home-Assistant instance at: {}", endpoint);

        let (mut ws, _) = connect_async(url)
            .await
            .context("failed to connect to Home-Assistant Web-Socket endpoint")?;

        if let Some(message) = ws.next().await {
            match message.context("failed to wait for the first Web-Socket message")? {
                WsMessage::Text(text) => {
                    let message: Message = serde_json::from_str(&text)
                        .context("failed to parse the first Web-Socket message")?;

                    println!("{:?}", message);
                }
                _ => {
                    return Err(anyhow::anyhow!("unexpected Web-Socket message"));
                }
            }
        } else {
            return Err(anyhow::anyhow!(
                "the stream closed while waiting for the first Web-Socket message"
            ));
        }

        Ok(Self { ws: Box::new(ws) })
    }

    pub async fn run(&mut self) -> Result<()> {
        while let Some(message) = self.ws.next().await {
            let message = message?;

            println!("{:?}", message);
        }

        Ok(())
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Message {
    AuthRequired { ha_version: String },
}
