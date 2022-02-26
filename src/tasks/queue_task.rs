use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::epsilon::api::epsilon_api::EpsilonApi;
use crate::epsilon::api::epsilon_events::EpsilonEvent::SendToServer;
use crate::epsilon::queue::queue_provider::QueueProvider;
use crate::epsilon::server::instance_type::InstanceType;
use crate::{EResult, InstanceProvider, Task};

pub struct QueueTask {
    epsilon_api: Arc<EpsilonApi>,
    instance_provider: Arc<InstanceProvider>,
    queue_provider: Arc<Mutex<QueueProvider>>,
}

#[async_trait]
impl Task for QueueTask {
    async fn init(
        epsilon_api: &Arc<EpsilonApi>,
        instance_provider: &Arc<InstanceProvider>,
        queue_provider: &Arc<Mutex<QueueProvider>>,
    ) -> EResult<Box<dyn Task>> {
        Ok(Box::new(Self {
            epsilon_api: Arc::clone(epsilon_api),
            queue_provider: Arc::clone(queue_provider),
            instance_provider: Arc::clone(instance_provider),
        }))
    }

    async fn run(&mut self) -> EResult<()> {
        for (template_name, queue) in self.queue_provider.lock().await.get_queues() {
            let template = queue.get_template();

            if !queue.is_empty() {
                let instance_starting = self
                    .instance_provider
                    .get_instances(&InstanceType::Server, Some(template_name))
                    .await?;

                let instance_ready = self
                    .instance_provider
                    .get_ready_instances(&InstanceType::Server, Some(template_name))
                    .await?;

                if instance_starting.is_empty() && instance_ready.is_empty() {
                    self.instance_provider.start_instance(template_name).await?;
                }

                if !instance_ready.is_empty() {
                    let instance = instance_ready.first().unwrap();
                    let mut available_slots =
                        template.slots as u32 - instance.get_info().await?.players.online;

                    while !queue.is_empty() && available_slots != 0 {
                        if let Some(group) = queue.pop() {
                            available_slots -= 1;
                            self.epsilon_api
                                .send(SendToServer(group, String::from(instance.get_name())))
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Queue:Task, check need to open instance queue"
    }
}
