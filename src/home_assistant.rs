use anyhow::{Context, Result};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use log::{info, warn};
use serde::{Deserialize, Serialize};
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
    access_token: String,
}

impl Client {
    pub async fn new(endpoint: &String, access_token: String) -> Result<Self> {
        let url = Url::parse(&format!("wss://{}/api/websocket", endpoint))
            .context("failed to parse home-assistant endpoint")?;

        info!("Connecting to Home-Assistant instance at: {}", endpoint);

        let (ws, _) = connect_async(url)
            .await
            .context("failed to connect to Home-Assistant Web-Socket endpoint")?;

        Ok(Self {
            ws: Box::new(ws),
            access_token,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            match self.read_message().await? {
                Message::AuthRequired { ha_version } => {
                    info!(
                        "Authenticating with Home-Assistant version {}...",
                        ha_version
                    );

                    self.send_message(Message::Auth {
                        access_token: self.access_token.clone(),
                    })
                    .await?;
                }
                Message::AuthOk { ha_version } => {
                    info!("Authenticated with Home-Assistant version {}", ha_version);
                }
                Message::AuthInvalid { message } => {
                    return Err(anyhow::anyhow!("Authentication failed: {}", message));
                }
                message => {
                    warn!(
                        "Unexpected message received while authenticating: {:?}",
                        message
                    );
                }
            }
        }
    }

    async fn read_message(&mut self) -> Result<Message> {
        loop {
            match self.ws.next().await {
                Some(Ok(message)) => match message {
                    WsMessage::Text(text) => {
                        break serde_json::from_str(&text)
                            .context("failed to parse the Web-Socket message")
                    }
                    WsMessage::Ping(data) => {
                        self.ws.send(WsMessage::Pong(data)).await?;
                        continue;
                    }
                    _ => {
                        break Err(anyhow::anyhow!(
                            "unexpected Web-Socket message: {:?}",
                            message
                        ))
                    }
                },
                Some(Err(err)) => break Err(err).context("failed to read the Web-Socket message"),
                None => {
                    break Err(anyhow::anyhow!(
                        "the stream closed while waiting for the first Web-Socket message"
                    ))
                }
            }
        }
    }

    async fn send_message(&mut self, message: Message) -> Result<()> {
        self.ws
            .send(
                serde_json::to_string(&message)
                    .context("failed to serialize the message")?
                    .into(),
            )
            .await
            .context("failed to send the Web-Socket message")
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Message {
    AuthRequired { ha_version: String },
    Auth { access_token: String },
    AuthOk { ha_version: String },
    AuthInvalid { message: String },
}
