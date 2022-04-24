use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::{DateTime, Utc};
use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use warp::{Filter, Rejection, Reply};

use crate::{
    config::HomeControlConfig,
    gpio_controller::GpioController,
    home_assistant::{self, Controller},
    Result,
};

pub struct Api {
    gpio_controller: Arc<GpioController>,
    ha_controller: Controller,
    home_control_config: HomeControlConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum Status {
    Disconnected,
    #[serde(rename_all = "camelCase")]
    Connected {
        location: String,
        weather_current: Box<WeatherStatus>,
        weather_forecast: Box<WeatherStatus>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherStatus {
    pub timestamp: DateTime<Utc>,
    pub state: String,
    pub humidity: Option<f64>,
    pub pressure: Option<f64>,
    pub temperature: f64,
    pub wind_speed: f64,
    pub wind_bearing: f64,
}

impl Status {
    fn new(
        ha_status: home_assistant::Status,
        home_control_config: &HomeControlConfig,
    ) -> Result<Self> {
        Ok(match ha_status {
            home_assistant::Status::Disconnected => Status::Disconnected,
            home_assistant::Status::Connected { mut entities } => {
                let weather_state: home_assistant::WeatherState = entities
                    .remove(&home_control_config.weather_entity)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Weather entity `{}` was not found",
                            home_control_config.weather_entity
                        )
                    })?
                    .try_into()?;

                let first_forecast = weather_state
                    .attributes
                    .forecast
                    .into_iter()
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("No forecast found"))?;

                let weather_current = Box::new(WeatherStatus {
                    timestamp: weather_state.last_changed,
                    state: weather_state.state,
                    humidity: Some(weather_state.attributes.humidity),
                    pressure: Some(weather_state.attributes.pressure),
                    temperature: weather_state.attributes.temperature,
                    wind_speed: weather_state.attributes.wind_speed,
                    wind_bearing: weather_state.attributes.wind_bearing,
                });
                let weather_forecast = Box::new(WeatherStatus {
                    timestamp: first_forecast.datetime,
                    state: first_forecast.condition,
                    humidity: None,
                    pressure: None,
                    temperature: first_forecast.temperature,
                    wind_speed: first_forecast.wind_speed,
                    wind_bearing: first_forecast.wind_bearing,
                });

                Status::Connected {
                    location: home_control_config.location.clone(),
                    weather_current,
                    weather_forecast,
                }
            }
        })
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ApiBool {
    Bool(bool),
    Integer(i8),
}

impl From<ApiBool> for bool {
    fn from(b: ApiBool) -> Self {
        match b {
            ApiBool::Bool(b) => b,
            ApiBool::Integer(i) => i != 0,
        }
    }
}

impl Api {
    pub fn new(
        gpio_controller: Arc<GpioController>,
        ha_controller: Controller,
        home_control_config: HomeControlConfig,
    ) -> anyhow::Result<Arc<Self>> {
        Ok(Arc::new(Self {
            gpio_controller,
            ha_controller,
            home_control_config,
        }))
    }

    pub async fn run(self: Arc<Self>) -> anyhow::Result<()> {
        let period = Duration::from_secs(1);
        let mut last_seen = Instant::now();
        let mut screen_status = false;

        loop {
            sleep(period).await;

            if self.gpio_controller.get_distance_cm().await?
                <= self.home_control_config.sensor_activation_distance_cm
            {
                last_seen = Instant::now();

                if !screen_status {
                    info!("Presence detected: turning on screen.");
                    screen_status = true;
                }
            } else if last_seen.elapsed() > self.home_control_config.presence_inactivity_timeout
                && screen_status
            {
                info!(
                    "Presence not detected for {:.2}s: turning off screen.",
                    self.home_control_config
                        .presence_inactivity_timeout
                        .as_secs_f64()
                );
                screen_status = false;
            }
        }
    }

    pub fn routes(
        self: &Arc<Self>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let api = Arc::clone(self);
        let api_filter = warp::any().map(move || Arc::clone(&api));

        // Status.
        let api_status_get = warp::path!("api" / "v1" / "status")
            .and(warp::get())
            .and(api_filter.clone())
            .and_then(Self::api_status_get);

        // Alarm.
        let api_alarm_get = warp::path!("api" / "v1" / "alarm")
            .and(warp::get())
            .and(api_filter.clone())
            .and_then(Self::api_alarm_get);

        // Light control.
        let api_light = warp::path!("api" / "v1" / "light" / String);

        let api_light_get = api_light
            .and(warp::get())
            .and(api_filter.clone())
            .and_then(|name, api: Arc<Api>| async move { api.api_light_get(name).await });

        let api_light_set = api_light
            .and(warp::post())
            .and(warp::body::content_length_limit(8))
            .and(api_filter)
            .and(warp::body::json())
            .and_then(|light: String, api: Arc<Api>, status| async move {
                Self::api_light_set(api, light, status).await
            });

        // Final path organization.
        api_status_get
            .or(api_alarm_get)
            .or(api_light_get)
            .or(api_light_set)
    }

    async fn api_status_get(self: Arc<Self>) -> Result<impl Reply, Rejection> {
        let ha_status = self.ha_controller.status().await;

        let status = match Status::new(ha_status, &self.home_control_config) {
            Ok(status) => status,
            Err(err) => {
                error!("failed to get status: {}", err);
                return Err(err.into());
            }
        };

        Ok(warp::reply::json(&status))
    }

    async fn api_alarm_get(self: Arc<Self>) -> Result<impl Reply, Rejection> {
        // TODO: Implement.
        //let status = self
        //    .ha_controller
        //    .get_light(GpioPin::RedLed)
        //    .map_err(|_| warp::reject::reject())?;
        let status = true;

        Ok(warp::reply::json(&status))
    }

    async fn api_light_get(self: Arc<Self>, _light: String) -> Result<impl Reply, Rejection> {
        let status = false;

        Ok(warp::reply::json(&status))
    }

    async fn api_light_set(
        self: Arc<Self>,
        light: String,
        status: ApiBool,
    ) -> Result<impl Reply, Rejection> {
        let status: bool = status.into();
        self.ha_controller
            .light_set(&format!("light.{}", light), status)
            .await
            .map_err(warp::reject::custom)?;

        Ok(warp::reply::json(&status))
    }
}
