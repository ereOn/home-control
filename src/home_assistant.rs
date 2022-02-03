use anyhow::{Context, Result};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use log::{debug, info, warn};
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
    tx: tokio::sync::mpsc::Sender<Message>,
    rx: tokio::sync::mpsc::Receiver<Message>,
}

pub struct Controller {
    tx: tokio::sync::mpsc::Sender<Message>,
}

impl Client {
    pub async fn new(endpoint: &String, access_token: String) -> Result<Self> {
        let url = Url::parse(&format!("wss://{}/api/websocket", endpoint))
            .context("failed to parse home-assistant endpoint")?;

        info!("Connecting to Home-Assistant instance at: {}", endpoint);

        let (ws, _) = connect_async(url)
            .await
            .context("failed to connect to Home-Assistant Web-Socket endpoint")?;

        let (tx, rx) = tokio::sync::mpsc::channel(1);

        Ok(Self {
            ws: Box::new(ws),
            access_token,
            tx,
            rx,
        })
    }

    /// Get a new controller on the client.
    pub fn new_controller(&self) -> Controller {
        Controller {
            tx: self.tx.clone(),
        }
    }

    /// Run the client and consumes it.
    pub async fn run(self) -> Result<()> {
        let mut rx = self.rx;
        let mut ws = self.ws;

        loop {
            tokio::select! {
                message = rx.recv() =>
                    if let Some(message) = message {
                        debug!("Sending message: {:?}", message);
                        //TODO: Increment messages ids...
                        Self::send_message(&mut ws, message).await?;
                    } else {
                        break Err(anyhow::anyhow!("channel closed"));
                    },
                message = Self::read_message(&mut ws) => match message? {
                    Message::AuthRequired { ha_version } => {
                        info!(
                            "Authenticating with Home-Assistant version {}...",
                            ha_version
                        );

                        Self::send_message(&mut ws, Message::Auth {
                            access_token: self.access_token.clone(),
                        })
                        .await?;
                    }
                    Message::AuthOk { ha_version } => {
                        info!("Authenticated with Home-Assistant version {}", ha_version);
                    }
                    Message::AuthInvalid { message } => {
                        break Err(anyhow::anyhow!("Authentication failed: {}", message));
                    }
                    message => {
                        warn!(
                            "Unexpected message received: {:?}",
                            message
                        );
                    }
                }
            }
        }
    }

    async fn read_message(mut ws: impl WebSocket) -> Result<Message> {
        loop {
            match ws.next().await {
                Some(Ok(message)) => match message {
                    WsMessage::Text(text) => match serde_json::from_str::<Message>(&text) {
                        Ok(message) => return Ok(message),
                        Err(err) => {
                            warn!("Failed to parse message `{:?}`: {}", text, err);
                            continue;
                        }
                    },
                    WsMessage::Ping(data) => {
                        ws.send(WsMessage::Pong(data)).await?;
                        continue;
                    }
                    _ => {
                        return Err(anyhow::anyhow!(
                            "unexpected Web-Socket message: {:?}",
                            message
                        ));
                    }
                },
                Some(Err(err)) => break Err(err).context("failed to read the Web-Socket message"),
                None => {
                    return Err(anyhow::anyhow!(
                        "the stream closed while waiting for the first Web-Socket message"
                    ));
                }
            };
        }
    }

    async fn send_message(mut ws: impl WebSocket, message: Message) -> Result<()> {
        ws.send(
            serde_json::to_string(&message)
                .context("failed to serialize the message")?
                .into(),
        )
        .await
        .context("failed to send the Web-Socket message")
    }
}

impl Controller {
    pub async fn call_service(
        &self,
        domain: &str,
        service: &str,
        service_data: Option<&serde_json::Value>,
        target: Option<&serde_json::Value>,
    ) -> Result<()> {
        self.tx
            .send(Message::CallService {
                id: 1,
                domain: domain.to_string(),
                service: service.to_string(),
                service_data: service_data.cloned(),
                target: target.cloned(),
            })
            .await
            .context("failed to send the call service message")
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Message {
    AuthRequired {
        ha_version: String,
    },
    Auth {
        access_token: String,
    },
    AuthOk {
        ha_version: String,
    },
    AuthInvalid {
        message: String,
    },
    CallService {
        id: u64,
        domain: String,
        service: String,
        service_data: Option<serde_json::Value>,
        target: Option<serde_json::Value>,
    },
}
