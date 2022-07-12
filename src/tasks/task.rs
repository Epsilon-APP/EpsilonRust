use crate::epsilon::api::epsilon_api::EpsilonApi;
use crate::epsilon::queue::queue_provider::QueueProvider;
use crate::{Context, EResult, InstanceProvider};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait Task: Send + Sync {
    async fn init(context: Arc<Context>) -> EResult<Box<dyn Task>>
    where
        Self: Sized;

    async fn run(&mut self) -> EResult<()>;

    fn get_name(&self) -> &'static str;
}
