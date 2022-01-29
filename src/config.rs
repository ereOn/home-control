use std::net::SocketAddr;

use clap::Parser;

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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(
        long,
        short,
        default_value = DEFAULT_LISTEN_ENDPOINT,
        value_name = "LISTEN_ENDPOINT"
    )]
    pub listen_endpoint: SocketAddr,

    #[clap(long, value_name = "REVERSE_PROXY_URL")]
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

        Ok(Self {
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
