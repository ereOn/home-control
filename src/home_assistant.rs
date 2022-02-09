use std::{collections::HashMap, fmt::Display, time::Duration};

use anyhow::Context;
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::Instant;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as WsError, Message as WsMessage},
};
use url::Url;

use crate::Result;

trait WebSocket<Item = WsMessage, Error = WsError>:
    Sink<Item, Error = Error> + Stream<Item = Result<Item, Error>> + Unpin
{
}

impl<T> WebSocket for T where
    T: Sink<WsMessage, Error = WsError> + Stream<Item = Result<WsMessage, WsError>> + Unpin
{
}

type Sender = tokio::sync::oneshot::Sender<Result<serde_json::Value>>;
type MessageAndSender = (Message, Sender);

pub struct Client {
    ws: Box<dyn WebSocket>,
    access_token: String,
    tx: tokio::sync::mpsc::Sender<MessageAndSender>,
    rx: tokio::sync::mpsc::Receiver<MessageAndSender>,
}

pub struct Controller {
    tx: tokio::sync::mpsc::Sender<MessageAndSender>,
}

impl Client {
    pub async fn new(endpoint: &str, access_token: String) -> Result<Self> {
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
        let mut id: u64 = 1;
        let mut senders_by_id = HashMap::new();

        loop {
            tokio::select! {
                pair = rx.recv() =>
                    if let Some((mut message, sender)) = pair {
                        if message.inject_id(id) {
                            senders_by_id.insert(id, sender);
                            id += 1;

                        debug!("Sending message: {:?}", message);
                        Self::send_message(&mut ws, message).await?;
                        } else {
                            warn!("Failed to inject message ID: not sending message: {:?}", message);
                        }
                    } else {
                        return Err(anyhow::anyhow!("channel closed")).map_err(Into::into);
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
                        return Err(anyhow::anyhow!("authentication failed: {}", message)).map_err(Into::into);
                    }
                    Message::Result { id, success, result, error } => {
                        let result = if success {
                            Ok(result)
                        } else {
                            Err(error.unwrap_or_default().into())
                        };

                        if let Some(sender) = senders_by_id.remove(&id) {
                            if sender.send(result).is_err() {
                                warn!("Failed to send result to sender for call #{}", id);
                            }
                        } else {
                            warn!("Discarding result for unknown id: {}", id);
                        }
                    }
                    Message::Pong { id } => {
                        if let Some(sender) = senders_by_id.remove(&id) {
                            if sender.send(Ok(json!(null))).is_err() {
                                warn!("Failed to send pong to sender for call #{}", id);
                            }
                        } else {
                            warn!("Discarding pong for unknown id: {}", id);
                        }
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
            break match ws.next().await {
                Some(Ok(message)) => match message {
                    WsMessage::Text(text) => match serde_json::from_str::<Message>(&text) {
                        Ok(message) => Ok(message),
                        Err(err) => {
                            warn!("Failed to parse message `{:?}`: {}", text, err);
                            continue;
                        }
                    },
                    WsMessage::Ping(data) => {
                        ws.send(WsMessage::Pong(data))
                            .await
                            .map_err::<anyhow::Error, _>(Into::into)?;
                        continue;
                    }
                    _ => Err(anyhow::anyhow!(
                        "unexpected Web-Socket message: {:?}",
                        message
                    ))
                    .map_err(Into::into),
                },
                Some(Err(err)) => Err(err)
                    .context("failed to read the Web-Socket message")
                    .map_err(Into::into),
                None => Err(anyhow::anyhow!(
                    "the stream closed while waiting for the first Web-Socket message"
                ))
                .map_err(Into::into),
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
        .map_err(Into::into)
    }
}

impl Controller {
    pub async fn ping(&self) -> Result<Duration> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        let now = Instant::now();

        self.tx
            .send((Message::Ping { id: 0 }, sender))
            .await
            .context("failed to send the ping message")?;

        receiver
            .await
            .context("failed to receive the ping response")??;

        let duration = now.elapsed();

        debug!("Ping duration: {}ms", duration.as_millis());

        Ok(duration)
    }

    pub async fn call_service(
        &self,
        domain: &str,
        service: &str,
        service_data: Option<&serde_json::Value>,
        target: Option<&serde_json::Value>,
    ) -> Result<()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        self.tx
            .send((
                Message::CallService {
                    id: 0,
                    domain: domain.to_string(),
                    service: service.to_string(),
                    service_data: service_data.cloned(),
                    target: target.cloned(),
                },
                sender,
            ))
            .await
            .context("failed to send the call service message")?;

        let result = receiver
            .await
            .context("failed to receive the call service response")??;

        debug!("Call service result: {:?}", result);

        Ok(())
    }

    pub async fn light_toggle(&self, entity_id: &str) -> Result<()> {
        self.call_service(
            "light",
            "toggle",
            Some(&json!({})),
            Some(&json!({ "entity_id": entity_id })),
        )
        .await
    }

    pub async fn subscribe_events(&self, event_type: Option<&str>) -> Result<()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        self.tx
            .send((
                Message::SubscribeEvents {
                    id: 0,
                    event_type: event_type.map(ToString::to_string),
                },
                sender,
            ))
            .await
            .context("failed to send the subscribe events message")?;

        let result = receiver
            .await
            .context("failed to receive the subscribe events response")??;

        debug!("Subscribe events result: {:?}", result);

        Ok(())
    }

    pub async fn subscribe_trigger(&self, trigger: serde_json::Value) -> Result<()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        self.tx
            .send((Message::SubscribeTrigger { id: 0, trigger }, sender))
            .await
            .context("failed to send the subscribe trigger message")?;

        let result = receiver
            .await
            .context("failed to receive the subscribe trigger response")??;

        debug!("Subscribe trigger result: {:?}", result);

        Ok(())
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
    Result {
        id: u64,
        success: bool,
        #[serde(default)]
        result: serde_json::Value,
        error: Option<Error>,
    },
    SubscribeEvents {
        id: u64,
        event_type: Option<String>,
    },
    SubscribeTrigger {
        id: u64,
        trigger: serde_json::Value,
    },
    Ping {
        id: u64,
    },
    Pong {
        id: u64,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    code: String,
    message: String,
}

impl Default for Error {
    fn default() -> Self {
        Self {
            code: "unspecified".to_string(),
            message: "no error information was present".to_string(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{}: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

impl Message {
    fn inject_id(&mut self, new_id: u64) -> bool {
        match self {
            Self::AuthRequired { .. }
            | Self::Auth { .. }
            | Self::AuthOk { .. }
            | Self::AuthInvalid { .. } => false,
            Self::CallService { id, .. }
            | Self::Result { id, .. }
            | Self::SubscribeEvents { id, .. }
            | Self::SubscribeTrigger { id, .. }
            | Self::Ping { id }
            | Self::Pong { id } => {
                *id = new_id;

                true
            }
        }
    }
}
