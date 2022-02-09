use anyhow::Result;
use log::info;
#[cfg(feature = "gpio")]
use rppal::{
    gpio::{Gpio, OutputPin},
    system::DeviceInfo,
};
#[cfg(not(feature = "gpio"))]
use std::{collections::HashMap, sync::Mutex};

use crate::config::GpioConfig;

pub struct GpioController {
    config: GpioConfig,
    #[cfg(feature = "gpio")]
    gpio: Gpio,
    #[cfg(not(feature = "gpio"))]
    gpio: Mutex<HashMap<u8, bool>>,
}

pub enum GpioPin {
    RedLed,
    GreenLed,
    Buzzer,
    Trigger,
    Echo,
}

impl GpioPin {
    fn into_pin_number(self, config: &GpioConfig) -> u8 {
        match self {
            GpioPin::RedLed => config.red_led_pin,
            GpioPin::GreenLed => config.green_led_pin,
            GpioPin::Buzzer => config.buzzer_pin,
            GpioPin::Trigger => config.trigger_pin,
            GpioPin::Echo => config.echo_pin,
        }
    }
}

#[cfg(feature = "gpio")]
impl GpioController {
    pub fn new(config: GpioConfig) -> Result<GpioController> {
        use anyhow::Context;

        let model = DeviceInfo::new()
            .context("failed to query Raspberry Pi model")?
            .model();

        info!("Raspberry Pi model: {}", model);

        let gpio = Gpio::new().context("failed to initialize GPIO")?;

        Ok(GpioController { config, gpio })
    }

    fn get_output_pin(&self, pin: u8) -> anyhow::Result<OutputPin> {
        Ok(self.gpio.get(pin)?.into_output())
    }

    pub fn get_output_pin_status(&self, pin: GpioPin) -> anyhow::Result<bool> {
        let pin = self.get_output_pin(pin.into_pin_number(&self.config))?;

        Ok(pin.is_set_high())
    }

    pub fn set_output_pin_status(&self, pin: GpioPin, status: bool) -> anyhow::Result<()> {
        let pin = pin.into_pin_number(&self.config);
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

#[cfg(not(feature = "gpio"))]
impl GpioController {
    pub fn new(config: GpioConfig) -> Result<GpioController> {
        info!("Running without GPIO support");

        let gpio = Mutex::new(HashMap::new());

        Ok(GpioController { config, gpio })
    }

    pub fn get_output_pin_status(&self, pin: GpioPin) -> anyhow::Result<bool> {
        let pin = pin.into_pin_number(&self.config);

        match self.gpio.lock() {
            Ok(gpio) => Ok(gpio.get(&pin).cloned().unwrap_or(false)),
            Err(err) => Err(anyhow::anyhow!("failed to acquire the GPIO lock: {}", err)),
        }
    }

    pub fn set_output_pin_status(&self, pin: GpioPin, status: bool) -> anyhow::Result<()> {
        match self.gpio.lock() {
            Ok(mut gpio) => {
                let pin = pin.into_pin_number(&self.config);

                info!("Setting pin {} to {}", pin, status);

                gpio.insert(pin, status);

                Ok(())
            }
            Err(err) => Err(anyhow::anyhow!("failed to acquire the GPIO lock: {}", err)),
        }
    }
}
