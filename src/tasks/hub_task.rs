use crate::epsilon::queue::queue_provider::QueueProvider;
use crate::epsilon::server::instance::VectorOfInstance;
use crate::epsilon::server::instance_type::InstanceType;
use crate::epsilon::server::state::EpsilonState;
use crate::epsilon::server::templates::template::Template;
use crate::{EResult, EpsilonApi, InstanceProvider, Task};
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

const DEFAULT_TIME: &'static u32 = &60;

pub struct HubTask {
    instance_provider: Arc<InstanceProvider>,
    hub_template: Template,

    time: u32,
}

#[async_trait]
impl Task for HubTask {
    async fn init(
        _epsilon_api: &Arc<EpsilonApi>,
        instance_provider: &Arc<InstanceProvider>,
        _queue_provider: &Arc<Mutex<QueueProvider>>,
    ) -> EResult<Box<dyn Task>> {
        Ok(Box::new(Self {
            instance_provider: Arc::clone(instance_provider),
            hub_template: instance_provider.get_template("hub").await?,

            time: 0,
        }))
    }

    async fn run(&mut self) -> EResult<()> {
        let template_name = &self.hub_template.name;

        let proxies = self
            .instance_provider
            .get_instances(&InstanceType::Proxy, None, Some(&EpsilonState::Running))
            .await?;

        let proxy_number = proxies.len();

        if proxy_number > 0 {
            let hubs_starting = self
                .instance_provider
                .get_instances(
                    &InstanceType::Server,
                    Some(template_name),
                    Some(&EpsilonState::Starting),
                )
                .await?;

            if hubs_starting.is_empty() {
                let hubs_ready = self
                    .instance_provider
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
                        self.instance_provider.start_instance(template_name).await?;
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
                            let info_result = instance.get_info().await;

                            if let Ok(info) = info_result {
                                let online_player = info.players.online;

                                if instance.get_state() == EpsilonState::Running
                                    && online_player <= n
                                {
                                    n = online_player;
                                    hub_option = Some(instance);
                                }
                            }
                        }

                        if let Some(hub) = hub_option {
                            let name = hub.get_name();

                            self.instance_provider.remove_instance(name).await?;

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
