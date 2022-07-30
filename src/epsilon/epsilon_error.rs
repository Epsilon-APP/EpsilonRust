use rocket::http::Status;
use rocket::response::Responder;
use rocket::{response, Request, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EpsilonError {
    #[error("API server error {0}")]
    ApiServerError(String),

    #[error("Failed to parse json")]
    ParseJsonError,

    #[error("Send event error {0}")]
    SendEventError(String),

    #[error("Create instance error, template is {0}")]
    CreateInstanceError(String),

    #[error("Remove instance error {0}")]
    RemoveInstanceError(String),

    #[error("Retrieve instance error")]
    RetrieveInstanceError,

    #[error("Retrieve status error")]
    RetrieveStatusError,

    #[error("Queue not found error {0}")]
    QueueNotFoundError(String),

    #[error("Request error {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Ping response error {0}")]
    PingMinecraftError(#[from] async_minecraft_ping::ServerError),

    #[error("Timeout error {0}")]
    TimeoutError(#[from] tokio::time::error::Elapsed),
}

impl<'r> Responder<'r, 'static> for EpsilonError {
    fn respond_to(self, _req: &'r Request<'_>) -> response::Result<'static> {
        Response::build().status(Status::InternalServerError).ok()
    }
}
