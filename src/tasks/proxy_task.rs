use crate::epsilon::queue::queue_provider::QueueProvider;
use crate::epsilon::server::instance_type::InstanceType;
use crate::epsilon::server::template::Template;
use crate::{EResult, EpsilonApi, InstanceProvider, Task};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ProxyTask {
    instance_provider: Arc<InstanceProvider>,
    proxy_template: Template,
}

#[async_trait]
impl Task for ProxyTask {
    async fn init(
        _epsilon_api: &Arc<EpsilonApi>,
        instance_provider: &Arc<InstanceProvider>,
        _queue_provider: &Arc<Mutex<QueueProvider>>,
    ) -> EResult<Box<dyn Task>> {
        Ok(Box::new(Self {
            instance_provider: Arc::clone(instance_provider),
            proxy_template: instance_provider.get_template("waterfall").await?,
        }))
    }

    async fn run(&mut self) -> EResult<()> {
        let template_name = &self.proxy_template.name;

        let proxies = self
            .instance_provider
            .get_instances(&InstanceType::Proxy, None)
            .await?;

        let number = proxies.len();

        if proxies.is_empty() {
            self.instance_provider.start_instance(template_name).await?;
        }

        if number > 1 {
            let proxy = proxies.first().unwrap();

            if proxy.is_ready() {
                let name = proxy.get_name();

                self.instance_provider.remove_instance(name).await?;
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Proxy:Task, check need to open a new proxy or close"
    }
}
