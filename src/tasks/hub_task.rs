use crate::epsilon::queue::queue_provider::QueueProvider;
use crate::epsilon::server::instance::VectorOfInstance;
use crate::epsilon::server::instance_type::InstanceType;
use crate::epsilon::server::template::Template;
use crate::{EResult, EpsilonApi, InstanceProvider, Task};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct HubTask {
    instance_provider: Arc<InstanceProvider>,
    hub_template: Template,
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
        }))
    }

    async fn run(&mut self) -> EResult<()> {
        let template_name = &self.hub_template.name;

        let proxies = self
            .instance_provider
            .get_ready_instances(&InstanceType::Proxy, None)
            .await?;

        let proxy_number = proxies.len();

        if proxy_number > 0 {
            let hubs = self
                .instance_provider
                .get_instances(&InstanceType::Server, Some(template_name))
                .await?;

            let hub_number = hubs.len() as u32;
            let hub_online_count = hubs.get_online_count().await?;

            let hub_necessary =
                ((hub_online_count as f32 * 1.6 / self.hub_template.slots as f32) + 1.0) as u32;

            if hub_number < hub_necessary {
                self.instance_provider.start_instance(template_name).await?;
            }

            if hub_number > hub_necessary {
                let mut n = 0;
                let mut hub_option = None;

                for instance in hubs {
                    let online_player = instance.get_info().await?.players.online;

                    if instance.is_ready() && online_player < n {
                        n = online_player;
                        hub_option = Some(instance);
                    }
                }

                if let Some(hub) = hub_option {
                    let name = hub.get_name();

                    self.instance_provider.remove_instance(name).await?;

                    info!("Hub {} is removed with {} online players", name, n);
                }
            }
        }

        Ok(())
    }

    fn get_name(&self) -> &'static str {
        "Hub:Task, check need to open a new hub or close"
    }
}
