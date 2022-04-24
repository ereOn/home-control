use anyhow::Result;
use log::info;
use std::sync::Arc;

#[cfg(feature = "gpio")]
use rppal::{
    gpio::{Gpio, InputPin, OutputPin, Trigger},
    system::DeviceInfo,
};

use crate::config::GpioConfig;

pub struct GpioController {
    #[cfg(feature = "gpio")]
    config: GpioConfig,
    #[cfg(feature = "gpio")]
    gpio: Gpio,
}

pub enum GpioPin {
    RedLed,
    GreenLed,
    Buzzer,
    Trigger,
    Echo,
}

#[cfg(feature = "gpio")]
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

    fn get_output_pin(&self, pin: GpioPin) -> anyhow::Result<OutputPin> {
        let pin = pin.into_pin_number(&self.config);
        Ok(self.gpio.get(pin)?.into_output())
    }

    fn get_input_pin(&self, pin: GpioPin) -> anyhow::Result<InputPin> {
        let pin = pin.into_pin_number(&self.config);
        Ok(self.gpio.get(pin)?.into_input())
    }

    fn set_output_pin_status(&self, pin: GpioPin, status: bool) -> anyhow::Result<()> {
        let status: bool = status.into();

        let mut pin = self.get_output_pin(pin)?;

        if status {
            pin.set_high();
        } else {
            pin.set_low();
        }

        Ok(())
    }

    fn compute_distance(&self) -> anyhow::Result<f64> {
        use anyhow::Context;

        let mut echo_pin = self.get_input_pin(GpioPin::Echo)?;

        // Set an interrupt *before* the trigger pin is set high.
        echo_pin
            .set_interrupt(Trigger::Both)
            .context("setting interrupt on pin")?;

        // Send out the signal.
        self.set_output_pin_status(GpioPin::Trigger, true)?;
        std::thread::sleep(std::time::Duration::from_micros(10));
        self.set_output_pin_status(GpioPin::Trigger, false)?;

        // Wait for the start of the echo to be received...
        echo_pin
            .poll_interrupt(false, Some(std::time::Duration::from_millis(10)))?
            .ok_or_else(|| anyhow::anyhow!("polling for rising edge timed out"))?;
        let start = std::time::Instant::now();

        echo_pin
            .poll_interrupt(false, Some(std::time::Duration::from_millis(10)))?
            .ok_or_else(|| anyhow::anyhow!("polling for falling edge timed out"))?;
        let stop = std::time::Instant::now();

        // Now compute the distance.
        let elapsed = stop.duration_since(start);

        // Sound travels at 343 m/s, which equates to 34300 * 10^-6 = 0.0343 cm/us.
        // Also divide by 2 because this is the duration of the round trip.
        //
        // Yay for physics!
        Ok(elapsed.as_micros() as f64 * 0.0343 / 2.0)
    }

    pub fn set_red_led(&self, status: bool) -> anyhow::Result<()> {
        info!("Setting red led to {}", status);

        self.set_output_pin_status(GpioPin::RedLed, status)
    }

    pub fn set_green_led(&self, status: bool) -> anyhow::Result<()> {
        info!("Setting green led to {}", status);

        self.set_output_pin_status(GpioPin::GreenLed, status)
    }

    pub fn set_buzzer(&self, status: bool) -> anyhow::Result<()> {
        info!("Setting buzzer to {}", status);

        self.set_output_pin_status(GpioPin::Buzzer, status)
    }
}

#[cfg(not(feature = "gpio"))]
impl GpioController {
    pub fn new(_config: GpioConfig) -> Result<GpioController> {
        info!("Running without GPIO support");

        Ok(GpioController {})
    }

    pub fn set_red_led(&self, status: bool) -> anyhow::Result<()> {
        info!("Setting red led to {}", status);

        Ok(())
    }

    pub fn set_green_led(&self, status: bool) -> anyhow::Result<()> {
        info!("Setting green led to {}", status);

        Ok(())
    }

    pub fn set_buzzer(&self, status: bool) -> anyhow::Result<()> {
        info!("Setting buzzer to {}", status);

        Ok(())
    }

    fn compute_distance(&self) -> anyhow::Result<f64> {
        Ok(0.0)
    }
}

impl GpioController {
    /// Get the distance in cm.
    pub async fn get_distance_cm(self: &Arc<Self>) -> anyhow::Result<f64> {
        let this = Arc::clone(self);
        tokio::task::spawn_blocking(move || this.compute_distance()).await?
    }
}
