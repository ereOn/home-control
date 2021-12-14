use std::sync::Arc;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

use crate::{
    config::GpioConfig,
    gpio_controller::{GpioController, GpioPin},
};

pub struct Api {
    gpio_controller: GpioController,
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
    pub fn new(config: GpioConfig) -> anyhow::Result<Arc<Self>> {
        let gpio = GpioController::new(config).context("failed to create GPIO")?;

        Ok(Arc::new(Self {
            gpio_controller: gpio,
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

        // Final path organization.
        api_red_led_get
            .or(api_red_led_set)
            .or(api_green_led_get)
            .or(api_green_led_set)
            .or(api_buzzer_get)
            .or(api_buzzer_set)
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
}
