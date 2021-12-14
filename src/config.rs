use std::net::SocketAddr;

use clap::{App, Arg, ArgMatches};

const ARG_LISTEN_ENDPOINT: &str = "listen-endpoint";
const ARG_REVERSE_PROXY_URL: &str = "reverse-proxy-url";
const ARG_RED_LED_PIN: &str = "red-led-pin";
const ARG_GREEN_LED_PIN: &str = "green-led-pin";
const ARG_BUZZER_PIN: &str = "buzzer-pin";
const ARG_TRIGGER_PIN: &str = "trigger-pin";
const ARG_ECHO_PIN: &str = "echo-pin";

const DEFAULT_LISTEN_ENDPOINT: &str = "127.0.0.1:8000";
const DEFAULT_RED_LED_PIN: &str = "17";
const DEFAULT_GREEN_LED_PIN: &str = "27";
const DEFAULT_BUZZER_PIN: &str = "18";
const DEFAULT_TRIGGER_PIN: &str = "24";
const DEFAULT_ECHO_PIN: &str = "23";

pub struct Config {
    pub listen_endpoint: SocketAddr,
    pub reverse_proxy_url: Option<String>,
    pub gpio_config: GpioConfig,
}

pub struct GpioConfig {
    pub red_led_pin: u8,
    pub green_led_pin: u8,
    pub buzzer_pin: u8,
    pub trigger_pin: u8,
    pub echo_pin: u8,
}

impl Config {
    pub fn new() -> anyhow::Result<Self> {
        fn simple_arg<'a>(name: &'a str, description: &'a str) -> Arg<'a, 'a> {
            Arg::with_name(name)
                .long(name)
                .help(description)
                .takes_value(true)
        }

        fn read_simple_arg<T>(matches: &ArgMatches, name: &str, default: &str) -> anyhow::Result<T>
        where
            T: std::str::FromStr,
            T::Err: Into<anyhow::Error>,
        {
            matches
                .value_of(name)
                .unwrap_or(default)
                .parse()
                .map_err(Into::into)
        }

        let matches = App::new("home-control")
            .version(env!("CARGO_PKG_VERSION"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .about(env!("CARGO_PKG_DESCRIPTION"))
            .arg(
                Arg::with_name(ARG_LISTEN_ENDPOINT)
                    .long(ARG_LISTEN_ENDPOINT)
                    .short("l")
                    .value_name("endpoint")
                    .help(&format!(
                        "The endpoint to listen on. Defaults to '{}'",
                        DEFAULT_LISTEN_ENDPOINT,
                    ))
                    .takes_value(true),
            )
            .arg(
                Arg::with_name(ARG_REVERSE_PROXY_URL)
                    .long(ARG_REVERSE_PROXY_URL)
                    .short("r")
                    .value_name("url")
                    .help("The URL to another server to proxy requests to. Default: does not use a proxy")
                    .takes_value(true),
            )
            .arg(simple_arg(ARG_RED_LED_PIN,  "The GPIO pin to use for the red LED"))
            .arg(simple_arg(ARG_GREEN_LED_PIN,  "The GPIO pin to use for the green LED"))
            .arg(simple_arg(ARG_BUZZER_PIN,  "The GPIO pin to use for the buzzer"))
            .arg(simple_arg(ARG_TRIGGER_PIN,  "The GPIO pin to use for the ultrasonic sensor trigger"))
            .arg(simple_arg(ARG_ECHO_PIN,  "The GPIO pin to use for the ultrasonic sensor echo"))
            .get_matches();

        Ok(Self {
            listen_endpoint: matches
                .value_of(ARG_LISTEN_ENDPOINT)
                .unwrap_or(DEFAULT_LISTEN_ENDPOINT)
                .parse()?,
            reverse_proxy_url: matches
                .value_of(ARG_REVERSE_PROXY_URL)
                .map(|s| s.to_string()),
            gpio_config: GpioConfig {
                red_led_pin: read_simple_arg(&matches, ARG_RED_LED_PIN, DEFAULT_RED_LED_PIN)?,
                green_led_pin: read_simple_arg(&matches, ARG_GREEN_LED_PIN, DEFAULT_GREEN_LED_PIN)?,
                buzzer_pin: read_simple_arg(&matches, ARG_BUZZER_PIN, DEFAULT_BUZZER_PIN)?,
                trigger_pin: read_simple_arg(&matches, ARG_TRIGGER_PIN, DEFAULT_TRIGGER_PIN)?,
                echo_pin: read_simple_arg(&matches, ARG_ECHO_PIN, DEFAULT_ECHO_PIN)?,
            },
        })
    }
}
