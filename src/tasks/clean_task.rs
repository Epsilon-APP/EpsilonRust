use crate::epsilon::queue::queue_provider::QueueProvider;
use crate::epsilon::server::instance_type::InstanceType;
use crate::{EResult, EpsilonApi, InstanceProvider, Task};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CleanTask {
    instance_provider: Arc<InstanceProvider>,
}

#[async_trait]
impl Task for CleanTask {
    async fn init(
        _epsilon_api: &Arc<EpsilonApi>,
        instance_provider: &Arc<InstanceProvider>,
        _queue_provider: &Arc<Mutex<QueueProvider>>,
    ) -> EResult<Box<dyn Task>> {
        Ok(Box::new(Self {
            instance_provider: Arc::clone(instance_provider),
        }))
    }

    async fn run(&mut self) -> EResult<()> {
        let servers = self
            .instance_provider
            .get_instances(&InstanceType::Server, None, None)
            .await?;

        let proxies = self
            .instance_provider
            .get_instances(&InstanceType::Proxy, None, None)
            .await?;

        for instance in servers {
            if instance.need_close() {
                let name = instance.get_name();

                self.instance_provider.remove_instance(name).await?;
            }
        }

        for instance in proxies {
            if instance.need_close() {
                let name = instance.get_name();

                self.instance_provider.remove_instance(name).await?;
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Clean:Task, clean all servers and proxies that need to be closed"
    }
}
