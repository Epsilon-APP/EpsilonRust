use std::sync::Arc;

use async_trait::async_trait;

use crate::{Context, EResult, Task};
use crate::controller::definitions::epsilon_instance::VectorOfInstance;
use crate::epsilon::api::common::epsilon_events::EpsilonEvent::SendToServer;
use crate::epsilon::server::instances::common::instance_type::InstanceType;
use crate::epsilon::server::instances::common::state::EpsilonState;

pub struct QueueTask {
    context: Arc<Context>,
}

#[async_trait]
impl Task for QueueTask {
    async fn init(context: Arc<Context>) -> EResult<Box<dyn Task>> {
        Ok(Box::new(Self { context }))
    }

    async fn run(&mut self) -> EResult<()> {
        let epsilon_api = self.context.get_epsilon_api();
        let instance_provider = self.context.get_instance_provider();
        let queue_provider = self.context.get_queue_provider();

        for (template_name, queue) in queue_provider.get_queues().into_iter() {
            if !queue.read().await.is_empty() {
                let instances_starting = instance_provider
                    .get_instances(
                        &InstanceType::Server,
                        Some(template_name),
                        Some(&EpsilonState::Starting),
                    )
                    .await?;

                let instances_ready = instance_provider
                    .get_instances(
                        &InstanceType::Server,
                        Some(template_name),
                        Some(&EpsilonState::Running),
                    )
                    .await?;

                if instances_starting.is_empty() && instances_ready.is_empty() {
                    instance_provider.start_instance(template_name).await?;
                    return Ok(());
                }

                let ready_available_slots_result = instances_ready.get_available_slots().await;

                if let Ok(ready_available_slots) = ready_available_slots_result {
                    if instances_starting.is_empty() && ready_available_slots < 1 {
                        instance_provider.start_instance(template_name).await?;
                    }
                }

                for instance in &instances_ready {
                    if let Ok(mut available_slots) = instance.get_available_slots().await {
                        while !queue.read().await.is_empty() && available_slots > 0 {
                            if let Some(group) = queue.write().await.pop() {
                                let group_size = group.players.len() as i32;

                                if group_size <= available_slots {
                                    available_slots -= group_size;

                                    epsilon_api.send(SendToServer(
                                        group,
                                        instance.metadata.name.as_ref().unwrap().clone(),
                                    ));
                                }
                            }
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
