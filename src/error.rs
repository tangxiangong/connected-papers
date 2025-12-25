#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum Error {
    #[error(transparent)]
    APIKeyNotFound(#[from] std::env::VarError),
    #[error(transparent)]
    SemanticScholarRequestFailed(#[from] crate::ss::client::RequestFailedError),
    #[error("{0}")]
    ReqwestError(String),
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::ReqwestError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
