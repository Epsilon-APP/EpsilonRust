use thiserror::Error;

#[derive(Error, Debug)]
pub enum EpsilonError {
    #[error("API server error -> {0}")]
    ApiServerError(String),
}
