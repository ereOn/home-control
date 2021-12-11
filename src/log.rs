use log::LevelFilter;
use simplelog::{ColorChoice, Config, TermLogger, TerminalMode};

pub fn init() {
    TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();
}
