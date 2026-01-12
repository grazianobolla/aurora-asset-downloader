use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("request error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Message(String),
}

impl From<&'static str> for AppError {
    fn from(value: &'static str) -> Self {
        AppError::Message(value.to_string())
    }
}
