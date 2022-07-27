use std::sync::Arc;

use async_trait::async_trait;

use crate::{Context, EResult, Task};
use crate::controller::definitions::epsilon_instance::VectorOfInstance;
use crate::epsilon::server::instances::common::instance_type::InstanceType;
use crate::epsilon::server::instances::common::state::EpsilonState;
use crate::epsilon::server::templates::template::Template;

const DEFAULT_TIME: &u32 = &60;

pub struct HubTask {
    context: Arc<Context>,

    hub_template: Template,
    time: u32,
}

#[async_trait]
impl Task for HubTask {
    async fn init(context: Arc<Context>) -> EResult<Box<dyn Task>> {
        let hub_template = context.get_template_provider().get_hub_template().await?;

        Ok(Box::new(Self {
            context,
            hub_template,

            time: 0,
        }))
    }

    async fn run(&mut self) -> EResult<()> {
        let instance_provider = self.context.get_instance_provider();
        let template_name = &self.hub_template.name;

        let proxies = instance_provider
            .get_instances(
                &InstanceType::Proxy,
                None,
                Some(&EpsilonState::Running),
            )
            .await?;

        let proxy_number = proxies.len();

        if proxy_number > 0 {
            let hubs_starting = instance_provider
                .get_instances(
                    &InstanceType::Server,
                    Some(template_name),
                    Some(&EpsilonState::Starting),
                )
                .await?;

            if hubs_starting.is_empty() {
                let hubs_ready = instance_provider
                    .get_instances(
                        &InstanceType::Server,
                        Some(template_name),
                        Some(&EpsilonState::Running),
                    )
                    .await?;

                if let Ok(hub_online_count) = hubs_ready.get_online_count().await {
                    let hub_number = hubs_ready.len() as u32;
                    let hub_necessary = ((hub_online_count as f32 * 1.6
                        / self.hub_template.slots as f32)
                        + 1.0) as u32;

                    if hub_number < hub_necessary {
                        instance_provider.start_instance(template_name).await?;
                    }

                    if hub_number > hub_necessary {
                        if self.time != *DEFAULT_TIME {
                            self.time += 1;

                            return Ok(());
                        } else {
                            self.time = 0;
                        }

                        let mut n = 0;
                        let mut hub_option = None;

                        for instance in hubs_ready {
                            let online_player_result = instance.get_online_count().await;

                            if let Ok(online_player) = online_player_result {
                                if instance.status.as_ref().unwrap().state == EpsilonState::Running
                                    && online_player <= n
                                {
                                    n = online_player;
                                    hub_option = Some(instance);
                                }
                            };
                        }

                        if let Some(hub) = hub_option {
                            let name = hub.metadata.name.as_ref().unwrap();

                            instance_provider.remove_instance(&name).await?;

                            info!("Hub {} is removed with {} online players", name, n);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Hub:Task, check need to open a new hub or close"
    }
}
