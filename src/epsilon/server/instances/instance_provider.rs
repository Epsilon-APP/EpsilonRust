use std::sync::Arc;

use kube::api::DeleteParams;
use kube::runtime::wait::await_condition;

use crate::controller::definitions::epsilon_instance::EpsilonInstance;
use crate::epsilon::server::instances::common::instance_type::InstanceType;
use crate::epsilon::server::instances::common::state::EpsilonState;
use crate::{EResult, EpsilonController};

pub struct InstanceProvider {
    epsilon_controller: Arc<EpsilonController>,
}

impl InstanceProvider {
    pub fn new(epsilon_controller: &Arc<EpsilonController>) -> InstanceProvider {
        Self {
            epsilon_controller: Arc::clone(epsilon_controller),
        }
    }

    pub async fn start_instance(&self, template_name: &str) -> EResult<EpsilonInstance> {
        Ok(self
            .epsilon_controller
            .create_epsilon_instance(template_name)
            .await?)
    }

    pub async fn remove_instance(&self, name: &str) -> EResult<()> {
        info!("An instance has been removed (name={})", name);

        self.epsilon_controller
            .get_epsilon_instance_api()
            .delete(name, &DeleteParams::default())
            .await?;

        Ok(())
    }

    pub async fn get_instance(&self, instance_name: &str) -> EResult<EpsilonInstance> {
        Ok(self
            .epsilon_controller
            .get_epsilon_instance_api()
            .get(instance_name)
            .await?)
    }

    pub async fn get_instances(
        &self,
        instance_type: &InstanceType,
        template_option: Option<&str>,
        state_option: Option<&EpsilonState>,
    ) -> EResult<Vec<Arc<EpsilonInstance>>> {
        let mut instances = self.epsilon_controller.get_epsilon_instance_store().state();

        for instance in &instances {
            let condition = await_condition(
                self.epsilon_controller.get_epsilon_instance_api().clone(),
                instance.metadata.name.as_ref().unwrap(),
                move |object: Option<&EpsilonInstance>| {
                    object.map_or(false, |instance| {
                        info!(
                            "Instance status ({}) : {}",
                            instance.metadata.name.as_ref().unwrap(),
                            instance.status.is_some()
                        );

                        instance.status.is_some()
                    })
                },
            );

            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), condition).await?;
        }

        for instance in &instances {
            info!(
                "Instance ({}) : {}",
                instance.metadata.name.as_ref().unwrap(),
                instance.status.is_some()
            );
        }

        instances = instances
            .into_iter()
            .filter(|instance| instance.status.as_ref().unwrap().t == *instance_type)
            .collect();

        if let Some(template_name) = template_option {
            instances = instances
                .into_iter()
                .filter(|instance| instance.spec.template == template_name)
                .collect();
        };

        if let Some(state) = state_option {
            instances = instances
                .into_iter()
                .filter(|instance| instance.status.as_ref().unwrap().state == *state)
                .collect();
        };

        Ok(instances)
    }

    pub async fn enable_in_game_instance(&self, name: &str) -> EResult<()> {
        self.epsilon_controller.in_game_epsilon_instance(name).await
    }
}
