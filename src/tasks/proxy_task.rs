use std::sync::Arc;

use async_trait::async_trait;

use crate::{Context, EResult, Task};
use crate::epsilon::server::instances::common::instance_type::InstanceType;
use crate::epsilon::server::templates::template::Template;

pub struct ProxyTask {
    context: Arc<Context>,
    proxy_template: Template,
}

#[async_trait]
impl Task for ProxyTask {
    async fn init(context: Arc<Context>) -> EResult<Box<dyn Task>> {
        info!("proxy");

        let proxy_template = context.get_template_provider().get_proxy_template().await?;

        Ok(Box::new(Self {
            context,
            proxy_template,
        }))
    }

    async fn run(&mut self) -> EResult<()> {
        let instance_provider = self.context.get_instance_provider();
        let template_name = &self.proxy_template.name;

        let proxies = instance_provider
            .get_instances(&InstanceType::Proxy, None, None)
            .await?;

        if proxies.is_empty() {
            instance_provider.start_instance(template_name).await?;
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Proxy:Task, check need to open a new proxy or close"
    }
}
