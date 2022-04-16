use std::{collections::HashMap, fmt::Display, time::Duration};

use anyhow::Context as _;
use chrono::{DateTime, Utc};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use log::{debug, error, info, warn};
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
    access_token: String,
    ws_url: Url,
    events_subscription: EventsSubscription,
    tx: tokio::sync::mpsc::Sender<MessageAndSender>,
    rx: tokio::sync::mpsc::Receiver<MessageAndSender>,
    events_tx: tokio::sync::broadcast::Sender<Box<Event>>,
    _events_rx: tokio::sync::broadcast::Receiver<Box<Event>>,
}

pub struct Controller {
    tx: tokio::sync::mpsc::Sender<MessageAndSender>,
    events_rx: tokio::sync::Mutex<tokio::sync::broadcast::Receiver<Box<Event>>>,
}

#[derive(Clone)]
pub enum EventsSubscription {
    All,
    Specific(Vec<String>),
}

impl EventsSubscription {
    fn to_event_types(&self) -> Vec<Option<String>> {
        match self {
            Self::All => vec![None],
            Self::Specific(event_types) => {
                event_types.iter().map(|s| Some(s.to_string())).collect()
            }
        }
    }
}

impl Client {
    pub async fn new(
        endpoint: &str,
        access_token: String,
        events_subscription: EventsSubscription,
    ) -> Result<Self> {
        info!("Using Home-Assistant instance at: {}", endpoint);

        let ws_url = Url::parse(&format!("wss://{}/api/websocket", endpoint))
            .context("failed to parse home-assistant endpoint")?;

        info!("Will establish Home-Assistant web-socket at: {}", ws_url);

        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let (events_tx, _events_rx) = tokio::sync::broadcast::channel(128);

        Ok(Self {
            access_token,
            ws_url,
            events_subscription,
            tx,
            rx,
            events_tx,
            _events_rx,
        })
    }

    async fn subscribe_to_events(
        tx: &mut tokio::sync::mpsc::Sender<MessageAndSender>,
        event_types: Vec<Option<String>>,
    ) -> Result<()> {
        info!("Subscribing to Home-Assistant events...");

        for event_type in event_types {
            let (sender, receiver) = tokio::sync::oneshot::channel();
            tx.send((Message::SubscribeEvents { id: 0, event_type }, sender))
                .await
                .context("failed to send the subscribe events message")?;

            let result = receiver
                .await
                .context("failed to receive the subscribe events response")??;

            debug!("Subscribe events result: {:?}", result);
        }

        info!("Subscribed to Home-Assistant events.");

        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(())
    }

    async fn subscribe_to_trigger(
        tx: &mut tokio::sync::mpsc::Sender<MessageAndSender>,
        trigger: serde_json::Value,
    ) -> Result<()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        tx.send((Message::SubscribeTrigger { id: 0, trigger }, sender))
            .await
            .context("failed to send the subscribe trigger message")?;

        let result = receiver
            .await
            .context("failed to receive the subscribe trigger response")??;

        debug!("Subscribe trigger result: {:?}", result);

        Ok(())
    }

    /// Get a new controller on the client.
    pub fn new_controller(&self) -> Controller {
        Controller {
            tx: self.tx.clone(),
            events_rx: tokio::sync::Mutex::new(self.events_tx.subscribe()),
        }
    }

    /// Run the client and consumes it.
    pub async fn run(mut self) -> Result<()> {
        let retry_delay = Duration::from_secs(5);

        loop {
            match connect_async(&self.ws_url).await {
                Err(err) => {
                    error!("Failed to establish web-socket to Home-Assistant: {}", err);
                    error!("Next attempt in {:.2}s...", retry_delay.as_secs());

                    tokio::time::sleep(retry_delay).await;
                }
                Ok((ws, _)) => {
                    if let Err(err) = self.run_with_ws(ws).await {
                        warn!(
                            "Home-Assistant web-socket connection was interuppted: {}",
                            err
                        );
                    }
                }
            }
        }
    }

    async fn run_with_ws(&mut self, mut ws: impl WebSocket) -> Result<()> {
        let mut authenticated = false;
        let event_types = self.events_subscription.to_event_types();
        let mut subscribed_to_events = event_types.is_empty();
        let mut id: u64 = 1;
        let mut senders_by_id = HashMap::new();
        let tx = &mut self.tx;
        let rx = &mut self.rx;

        let subscription = Self::subscribe_to_events(tx, event_types.clone());
        tokio::pin!(subscription);

        loop {
            tokio::select! {
                _ = &mut subscription, if authenticated && !subscribed_to_events => {
                    subscribed_to_events = true;
                }
                pair = rx.recv(), if authenticated =>
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
                        authenticated = true;
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
                    Message::Event { id, event } => {
                        debug!("Received event {}: {}", id, event);

                        self.events_tx
                            .send(event)
                            .context("failed to send event").map(|_| ())?;
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
                    WsMessage::Close(Some(frame)) => {
                        return Err(anyhow::anyhow!(
                            "Home-Assistant web-socket closed with code {}: {}",
                            frame.code,
                            frame.reason
                        ))
                        .map_err(Into::into);
                    }
                    _ => Err(anyhow::anyhow!(
                        "unexpected Home-Assistant web-socket message: {:?}",
                        message
                    ))
                    .map_err(Into::into),
                },
                Some(Err(err)) => Err(err)
                    .context("failed to read the web-socket message")
                    .map_err(Into::into),
                None => Err(anyhow::anyhow!(
                    "the stream closed while waiting for the first web-socket message from Home-Assistant"
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

    pub async fn light_set(&self, entity_id: &str, status: bool) -> Result<()> {
        self.call_service(
            "light",
            if status { "turn_on" } else { "turn_off" },
            Some(&json!({})),
            Some(&json!({ "entity_id": entity_id })),
        )
        .await
    }

    /// Wait for the next event.
    ///
    /// # Errors
    ///
    /// Returns an error if the event stream closed.
    pub async fn wait_for_event(&self) -> Result<Box<Event>> {
        self.events_rx
            .lock()
            .await
            .recv()
            .await
            .context("failed to receive event")
            .map_err(Into::into)
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
    Event {
        id: u64,
        event: Box<Event>,
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
            | Self::Pong { id }
            | Self::Event { id, .. } => {
                *id = new_id;

                true
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum Event {
    StateChanged {
        context: Context,
        data: StateChangedData,
        origin: String,
        time_fired: DateTime<Utc>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Context {
    pub id: String,
    pub parent_id: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StateChangedData {
    pub entity_id: String,
    pub old_state: Option<State>,
    pub new_state: Option<State>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    pub entity_id: String,
    pub attributes: serde_json::Value,
    pub context: Context,
    pub last_changed: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub state: String,
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::StateChanged {
                context: _,
                data,
                origin: _,
                time_fired: _,
            } => write!(
                f,
                "{}: {} -> {}",
                data.entity_id,
                data.old_state
                    .as_ref()
                    .map(|s| s.state.as_str())
                    .unwrap_or_default(),
                data.new_state
                    .as_ref()
                    .map(|s| s.state.as_str())
                    .unwrap_or_default(),
            ),
        }
    }
}

impl State {
    pub fn as_bool(&self) -> bool {
        matches!(self.state.as_str(), "on" | "1" | "true")
    }
}
