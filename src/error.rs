#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("home assistant error: {error}")]
    HomeAssistantError {
        #[from]
        error: super::home_assistant::Error,
    },
    #[error("unknown error: {source}")]
    Unknown {
        #[from]
        source: anyhow::Error,
    },
}

impl warp::reject::Reject for Error {}

pub type Result<T, E = Error> = std::result::Result<T, E>;
