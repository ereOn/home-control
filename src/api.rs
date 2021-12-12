use std::sync::Arc;

use anyhow::Context;
use log::info;
use rppal::{
    gpio::{Gpio, OutputPin},
    system::DeviceInfo,
};
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};

use crate::config::ApiConfig;

pub struct Api {
    config: ApiConfig,
    gpio: Gpio,
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
    pub fn new(config: ApiConfig) -> anyhow::Result<Arc<Self>> {
        let model = DeviceInfo::new()
            .context("Failed to query Raspberry Pi model")?
            .model();

        info!("Raspberry Pi model: {}", model);

        let gpio = Gpio::new().context("Failed to create GPIO")?;

        Ok(Arc::new(Self { config, gpio }))
    }

    pub fn config(&self) -> &ApiConfig {
        &self.config
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
            .get_output_pin_status(self.config.buzzer_pin)
            .map_err(|_| warp::reject::reject())?;

        Ok(warp::reply::json(&status))
    }

    async fn api_buzzer_set(self: Arc<Self>, status: ApiBool) -> Result<impl Reply, Rejection> {
        let status = status.into();
        self.set_output_pin_status(self.config.buzzer_pin, status)
            .map_err(|_| warp::reject::reject())?;

        Ok(warp::reply::json(&status))
    }

    async fn api_green_led_get(self: Arc<Self>) -> Result<impl Reply, Rejection> {
        let status = self
            .get_output_pin_status(self.config.green_led_pin)
            .map_err(|_| warp::reject::reject())?;

        Ok(warp::reply::json(&status))
    }

    async fn api_green_led_set(self: Arc<Self>, status: ApiBool) -> Result<impl Reply, Rejection> {
        let status = status.into();
        self.set_output_pin_status(self.config.green_led_pin, status)
            .map_err(|_| warp::reject::reject())?;

        Ok(warp::reply::json(&status))
    }

    async fn api_red_led_get(self: Arc<Self>) -> Result<impl Reply, Rejection> {
        let status = self
            .get_output_pin_status(self.config.red_led_pin)
            .map_err(|_| warp::reject::reject())?;

        Ok(warp::reply::json(&status))
    }

    async fn api_red_led_set(self: Arc<Self>, status: ApiBool) -> Result<impl Reply, Rejection> {
        let status = status.into();
        self.set_output_pin_status(self.config.red_led_pin, status)
            .map_err(|_| warp::reject::reject())?;

        Ok(warp::reply::json(&status))
    }

    fn get_output_pin(&self, pin: u8) -> anyhow::Result<OutputPin> {
        Ok(self.gpio.get(pin)?.into_output())
    }

    fn get_output_pin_status(&self, pin: u8) -> anyhow::Result<bool> {
        let pin = self.get_output_pin(pin)?;

        Ok(pin.is_set_high())
    }

    fn set_output_pin_status(&self, pin: u8, status: bool) -> anyhow::Result<()> {
        let status: bool = status.into();

        info!("Setting pin {} to {}", pin, status);

        let mut pin = self.get_output_pin(pin)?;

        if status {
            pin.set_high();
        } else {
            pin.set_low();
        }

        Ok(())
    }
}
