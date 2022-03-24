use crate::epsilon::api::epsilon_events::EpsilonEvent;
use crate::epsilon::queue::queue_provider::QueueProvider;
use crate::epsilon::server::instance_type::InstanceType;
use crate::{EResult, EpsilonApi, InstanceProvider, Task};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct CleanTask {
    epsilon_api: Arc<EpsilonApi>,
    instance_provider: Arc<InstanceProvider>,
}

#[async_trait]
impl Task for CleanTask {
    async fn init(
        epsilon_api: &Arc<EpsilonApi>,
        instance_provider: &Arc<InstanceProvider>,
        _queue_provider: &Arc<Mutex<QueueProvider>>,
    ) -> EResult<Box<dyn Task>> {
        Ok(Box::new(Self {
            epsilon_api: Arc::clone(epsilon_api),
            instance_provider: Arc::clone(instance_provider),
        }))
    }

    async fn run(&mut self) -> EResult<()> {
        let servers = self
            .instance_provider
            .get_instances(&InstanceType::Server, None, None)
            .await
            .unwrap();

        let proxies = self
            .instance_provider
            .get_instances(&InstanceType::Proxy, None, None)
            .await
            .unwrap();

        for instance in servers {
            if instance.need_close() {
                let name = instance.get_name();
                let event = EpsilonEvent::ClearServer(name.to_string());

                self.epsilon_api.send(event);
                self.instance_provider.remove_instance(name).await.unwrap();

                info!("Cleaned server: {}", name);
            }
        }

        for instance in proxies {
            if instance.need_close() {
                let name = instance.get_name();
                let event = EpsilonEvent::ClearServer(name.to_string());

                self.epsilon_api.send(event);
                self.instance_provider.remove_instance(name).await.unwrap();

                info!("Clean proxy: {}", name);
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Clean:Task, clean all servers and proxies that need to be closed"
    }
}
