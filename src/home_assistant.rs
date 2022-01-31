use anyhow::{Context, Result};
use futures_util::{Sink, Stream, StreamExt};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as WsError, Message},
};
use url::Url;

trait WebSocket<Item = Message, Error = WsError>:
    Sink<Item, Error = Error> + Stream<Item = Result<Item, Error>> + Unpin
{
}

impl<T> WebSocket for T where
    T: Sink<Message, Error = WsError> + Stream<Item = Result<Message, WsError>> + Unpin
{
}

pub struct Client {
    ws: Box<dyn WebSocket>,
}

impl Client {
    pub async fn new(endpoint: String) -> Result<Self> {
        let url = Url::parse(&format!("wss://{}/api/websocket", endpoint))
            .context("failed to parse home-assistant endpoint")?;

        let (ws, _) = connect_async(url).await?;

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
