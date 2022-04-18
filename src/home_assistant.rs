use std::{collections::HashMap, fmt::Display, sync::Arc, time::Duration};

use anyhow::Context as _;
use chrono::{DateTime, Utc};
use futures_util::{Sink, SinkExt, Stream, StreamExt};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Status {
    Connected { entities: HashMap<String, State> },
    Disconnected,
}

pub struct Client {
    access_token: String,
    ws_url: Url,
    events_subscription: Vec<Option<String>>,
    tx: tokio::sync::mpsc::Sender<MessageAndSender>,
    rx: tokio::sync::mpsc::Receiver<MessageAndSender>,
    status: Arc<RwLock<Status>>,
}

pub struct Controller {
    tx: tokio::sync::mpsc::Sender<MessageAndSender>,
    status: Arc<RwLock<Status>>,
}

impl Client {
    pub async fn new(endpoint: &str, access_token: String) -> Result<Self> {
        info!("Using Home-Assistant instance at: {}", endpoint);

        let ws_url = Url::parse(&format!("wss://{}/api/websocket", endpoint))
            .context("failed to parse home-assistant endpoint")?;

        info!("Will establish Home-Assistant web-socket at: {}", ws_url);

        let (tx, rx) = tokio::sync::mpsc::channel(1);

        let events_subscription = vec![Some("state_changed".to_string())];

        Ok(Self {
            access_token,
            ws_url,
            events_subscription,
            tx,
            rx,
            status: Arc::new(RwLock::new(Status::Disconnected)),
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

    async fn _subscribe_to_trigger(
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

    async fn get_states(
        tx: &mut tokio::sync::mpsc::Sender<MessageAndSender>,
    ) -> Result<HashMap<String, State>> {
        let (sender, receiver) = tokio::sync::oneshot::channel();

        tx.send((Message::GetStates { id: 0 }, sender))
            .await
            .context("failed to send the `get states` message")?;

        let result = receiver
            .await
            .context("failed to receive the `get states` response")??;

        debug!("Get states result: {:?}", result);

        Ok(serde_json::from_value::<Vec<State>>(result)?
            .into_iter()
            .map(|s| (s.entity_id.to_string(), s))
            .collect())
    }

    /// Get a new controller on the client.
    pub fn new_controller(&self) -> Controller {
        Controller {
            tx: self.tx.clone(),
            status: Arc::clone(&self.status),
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
                        *self.status.write().await = Status::Disconnected;

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
        let mut init_done = false;
        let mut id: u64 = 1;
        let mut senders_by_id = HashMap::new();
        let tx = &mut self.tx;
        let rx = &mut self.rx;

        async fn init_fn(
            tx: &mut tokio::sync::mpsc::Sender<MessageAndSender>,
            event_types: Vec<Option<String>>,
        ) -> Result<HashMap<String, State>> {
            Client::subscribe_to_events(tx, event_types).await?;

            Client::get_states(tx).await
        }

        let init = init_fn(tx, self.events_subscription.clone());

        tokio::pin!(init);

        let mut last_ping = tokio::time::Instant::now();
        let mut last_ping_id = id;
        let ping_interval = Duration::from_secs(10);

        loop {
            tokio::select! {
                states = &mut init, if authenticated && !init_done => {
                    init_done = true;
                    *self.status.write().await = Status::Connected{entities: states?};
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
                _ = tokio::time::sleep_until(last_ping + ping_interval), if authenticated => {
                    last_ping = tokio::time::Instant::now();

                    id += 1;
                    last_ping_id = id;
                    Self::send_message(&mut ws, Message::Ping { id }).await?;
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
                        if id == last_ping_id  {
                            let duration = last_ping.elapsed();

                            debug!("Ping duration: {}ms", duration.as_millis());
                        } else {
                            warn!("Discarding unexpected pong with id `{}` when `{}` was expected", id, last_ping_id);
                        }
                    }
                    Message::Event { id, event } => {
                        debug!("Received event {}: {}", id, event);

                        if let Event::StateChanged {
                                data: StateChangedData {
                                    entity_id,
                                    new_state: Some(new_state),
                                    ..
                                },
                            ..
                        } = event.as_ref() {
                            if let Status::Connected{entities} = &mut *self.status.write().await {
                                entities.insert(entity_id.clone(), new_state.clone());
                            }
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
    pub async fn status(&self) -> Status {
        (*self.status.read().await).clone()
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
    GetStates {
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
            | Self::Pong { id }
            | Self::Event { id, .. }
            | Self::GetStates { id } => {
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

impl From<State> for bool {
    fn from(s: State) -> Self {
        matches!(s.state.as_str(), "on" | "1" | "true")
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeatherState {
    pub entity_id: String,
    pub state: String,
    pub last_changed: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub attributes: WeatherAttributes,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeatherAttributes {
    pub attribution: Option<String>,
    pub forecast: Vec<WeatherForecast>,
    pub friendly_name: String,
    pub humidity: f64,
    pub pressure: f64,
    pub temperature: f64,
    pub wind_bearing: f64,
    pub wind_speed: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WeatherForecast {
    pub condition: String,
    pub datetime: DateTime<Utc>,
    pub precipitation: f64,
    pub temperature: f64,
    pub templow: f64,
    pub wind_bearing: f64,
    pub wind_speed: f64,
}

impl TryFrom<State> for WeatherState {
    type Error = crate::Error;

    fn try_from(value: State) -> Result<Self, Self::Error> {
        Ok(WeatherState {
            entity_id: value.entity_id,
            state: value.state,
            last_changed: value.last_changed,
            last_updated: value.last_updated,
            attributes: serde_json::from_value(value.attributes)?,
        })
    }
}
