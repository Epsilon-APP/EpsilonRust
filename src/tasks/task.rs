use std::sync::Arc;

use async_trait::async_trait;

use crate::{Context, EResult};

#[async_trait]
pub trait Task: Send + Sync {
    async fn init(context: Arc<Context>) -> EResult<Box<dyn Task>>
    where
        Self: Sized;

    async fn run(&mut self) -> EResult<()>;

    fn get_name(&self) -> &'static str;
}
