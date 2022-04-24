use std::{net::SocketAddr, path::PathBuf, time::Duration};

use clap::Parser;
use serde::Deserialize;
use serde_with::{serde_as, DurationSeconds};

const DEFAULT_RED_LED_PIN: &str = "17";
const DEFAULT_GREEN_LED_PIN: &str = "27";
const DEFAULT_BUZZER_PIN: &str = "18";
const DEFAULT_TRIGGER_PIN: &str = "24";
const DEFAULT_ECHO_PIN: &str = "23";

pub struct Config {
    pub debug: bool,
    pub home_control_config: HomeControlConfig,
    pub listen_endpoint: SocketAddr,
    pub reverse_proxy_url: Option<String>,
    pub gpio_config: GpioConfig,
    pub home_assistant_endpoint: String,
    pub home_assistant_token: String,
}

pub struct GpioConfig {
    pub red_led_pin: u8,
    pub green_led_pin: u8,
    pub buzzer_pin: u8,
    pub trigger_pin: u8,
    pub echo_pin: u8,
}

/// The configuration for the home-control application.
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct HomeControlConfig {
    /// The location to display in the UI.
    pub location: String,

    /// The entity to fetch the weather from.
    pub weather_entity: String,

    /// Sensor activation distance.
    #[serde(default = "HomeControlConfig::default_sensor_activation_distance")]
    pub sensor_activation_distance_cm: f64,

    /// Presence inactivity timeout.
    ///
    /// The time in seconds to wait after the sensor has detected an absence to trigger a reaction.
    #[serde(default = "HomeControlConfig::default_presence_inactivity_timeout")]
    #[serde_as(as = "DurationSeconds<f64>")]
    pub presence_inactivity_timeout: Duration,
}

impl HomeControlConfig {
    fn default_sensor_activation_distance() -> f64 {
        40.0
    }

    fn default_presence_inactivity_timeout() -> Duration {
        Duration::from_secs(5)
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Enables debug output.
    #[clap(long, short)]
    pub debug: bool,

    #[clap(
        long,
        value_name = "CONFIG_FILE",
        env,
        help = "The path to the configuration file",
        default_value = "/etc/home-control/config.yaml"
    )]
    pub config_file: PathBuf,

    #[clap(
        value_name = "HOME_ASSISTANT_ENDPOINT",
        env,
        help = "The endpoint of the Home Assistant API. Example: `host:port`"
    )]
    pub home_assistant_endpoint: String,

    #[clap(
        long,
        short = 't',
        env,
        value_name = "HOME_ASSISTANT_TOKEN",
        help = "The Home Assistant API long-lived token"
    )]
    pub home_assistant_token: String,

    #[clap(
        long,
        short,
        default_value = "127.0.0.1:8000",
        value_name = "LISTEN_ENDPOINT"
    )]
    pub listen_endpoint: SocketAddr,

    #[clap(long, short, value_name = "REVERSE_PROXY_URL")]
    pub reverse_proxy_url: Option<String>,

    #[clap(
        long,
        default_value = DEFAULT_RED_LED_PIN,
        value_name = "RED_LED_PIN"
    )]
    pub red_led_pin: u8,

    #[clap(
        long,
        default_value = DEFAULT_GREEN_LED_PIN,
        value_name = "GREEN_LED_PIN"
    )]
    pub green_led_pin: u8,

    #[clap(
        long,
        default_value = DEFAULT_BUZZER_PIN,
        value_name = "BUZZER_PIN"
    )]
    pub buzzer_pin: u8,

    #[clap(
        long,
        default_value = DEFAULT_TRIGGER_PIN,
        value_name = "TRIGGER_PIN"
    )]
    pub trigger_pin: u8,

    #[clap(
        long,
        default_value = DEFAULT_ECHO_PIN,
        value_name = "ECHO_PIN"
    )]
    pub echo_pin: u8,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        let args = Args::try_parse()?;
        let config_file = args.config_file;
        let home_control_config = config::Config::builder()
            .add_source(config::File::from(config_file))
            .add_source(config::Environment::with_prefix("HOME_CONTROL"))
            .build()?
            .try_deserialize()?;

        Ok(Self {
            debug: args.debug,
            home_control_config,
            home_assistant_endpoint: args.home_assistant_endpoint,
            home_assistant_token: args.home_assistant_token,
            listen_endpoint: args.listen_endpoint,
            reverse_proxy_url: args.reverse_proxy_url,
            gpio_config: GpioConfig {
                red_led_pin: args.red_led_pin,
                green_led_pin: args.green_led_pin,
                buzzer_pin: args.buzzer_pin,
                trigger_pin: args.trigger_pin,
                echo_pin: args.echo_pin,
            },
        })
    }
}
