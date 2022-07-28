use std::sync::Arc;

use async_trait::async_trait;

use crate::epsilon::epsilon_error::EpsilonError;
use crate::Context;

#[async_trait]
pub trait Task: Send + Sync {
    async fn init(context: Arc<Context>) -> Result<Box<dyn Task>, EpsilonError>
    where
        Self: Sized;

    async fn run(&mut self) -> Result<(), EpsilonError>;

    fn get_name(&self) -> &'static str;
}
