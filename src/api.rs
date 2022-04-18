use std::sync::Arc;

use anyhow::Context;
use chrono::{DateTime, Utc};
use log::error;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

use crate::{
    config::{GpioConfig, HomeControlConfig},
    gpio_controller::{GpioController, GpioPin},
    home_assistant::{self, Controller},
    Result,
};

pub struct Api {
    gpio_controller: GpioController,
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
        config: GpioConfig,
        ha_controller: Controller,
        home_control_config: HomeControlConfig,
    ) -> anyhow::Result<Arc<Self>> {
        let gpio = GpioController::new(config).context("failed to create GPIO")?;

        Ok(Arc::new(Self {
            gpio_controller: gpio,
            ha_controller,
            home_control_config,
        }))
    }

    pub fn routes(
        self: &Arc<Self>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let api = Arc::clone(self);
        let api_filter = warp::any().map(move || Arc::clone(&api));

        // Buzzer control.
        let api_buzzer = warp::path!("api" / "v1" / "buzzer");

        let api_buzzer_get = api_buzzer
            .and(warp::get())
            .and(api_filter.clone())
            .and_then(Self::api_buzzer_get);

        let api_buzzer_set = api_buzzer
            .and(warp::post())
            .and(warp::body::content_length_limit(8))
            .and(api_filter.clone())
            .and(warp::body::json())
            .and_then(Self::api_buzzer_set);

        // Green LED control.
        let api_green_led = warp::path!("api" / "v1" / "led" / "green");

        let api_green_led_get = api_green_led
            .and(warp::get())
            .and(api_filter.clone())
            .and_then(Self::api_green_led_get);

        let api_green_led_set = api_green_led
            .and(warp::post())
            .and(warp::body::content_length_limit(8))
            .and(api_filter.clone())
            .and(warp::body::json())
            .and_then(Self::api_green_led_set);

        // Red LED control.
        let api_red_led = warp::path!("api" / "v1" / "led" / "red");

        let api_red_led_get = api_red_led
            .and(warp::get())
            .and(api_filter.clone())
            .and_then(Self::api_red_led_get);

        let api_red_led_set = api_red_led
            .and(warp::post())
            .and(warp::body::content_length_limit(8))
            .and(api_filter.clone())
            .and(warp::body::json())
            .and_then(Self::api_red_led_set);

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
        api_red_led_get
            .or(api_red_led_set)
            .or(api_green_led_get)
            .or(api_green_led_set)
            .or(api_buzzer_get)
            .or(api_buzzer_set)
            .or(api_status_get)
            .or(api_alarm_get)
            .or(api_light_get)
            .or(api_light_set)
    }

    async fn api_buzzer_get(self: Arc<Self>) -> Result<impl Reply, Rejection> {
        let status = self
            .gpio_controller
            .get_output_pin_status(GpioPin::Buzzer)
            .map_err(|_| warp::reject::reject())?;

        Ok(warp::reply::json(&status))
    }

    async fn api_buzzer_set(self: Arc<Self>, status: ApiBool) -> Result<impl Reply, Rejection> {
        let status = status.into();
        self.gpio_controller
            .set_output_pin_status(GpioPin::Buzzer, status)
            .map_err(|_| warp::reject::reject())?;

        Ok(warp::reply::json(&status))
    }

    async fn api_green_led_get(self: Arc<Self>) -> Result<impl Reply, Rejection> {
        let status = self
            .gpio_controller
            .get_output_pin_status(GpioPin::GreenLed)
            .map_err(|_| warp::reject::reject())?;

        Ok(warp::reply::json(&status))
    }

    async fn api_green_led_set(self: Arc<Self>, status: ApiBool) -> Result<impl Reply, Rejection> {
        let status = status.into();
        self.gpio_controller
            .set_output_pin_status(GpioPin::GreenLed, status)
            .map_err(|_| warp::reject::reject())?;

        Ok(warp::reply::json(&status))
    }

    async fn api_red_led_get(self: Arc<Self>) -> Result<impl Reply, Rejection> {
        let status = self
            .gpio_controller
            .get_output_pin_status(GpioPin::RedLed)
            .map_err(|_| warp::reject::reject())?;

        Ok(warp::reply::json(&status))
    }

    async fn api_red_led_set(self: Arc<Self>, status: ApiBool) -> Result<impl Reply, Rejection> {
        let status = status.into();
        self.gpio_controller
            .set_output_pin_status(GpioPin::RedLed, status)
            .map_err(|_| warp::reject::reject())?;

        Ok(warp::reply::json(&status))
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
