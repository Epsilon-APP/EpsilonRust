use std::sync::Arc;

use kube::api::DeleteParams;
use serde_json::{Map, Value};

use crate::controller::definitions::epsilon_instance::EpsilonInstance;
use crate::epsilon::epsilon_error::EpsilonError;
use crate::epsilon::server::instances::common::instance_type::InstanceType;
use crate::epsilon::server::instances::common::state::EpsilonState;
use crate::EpsilonController;

pub struct InstanceProvider {
    epsilon_controller: Arc<EpsilonController>,
}

impl InstanceProvider {
    pub fn new(epsilon_controller: &Arc<EpsilonController>) -> InstanceProvider {
        Self {
            epsilon_controller: Arc::clone(epsilon_controller),
        }
    }

    pub async fn start_instance(
        &self,
        template_name: &str,
        content: Option<Value>,
    ) -> Result<EpsilonInstance, EpsilonError> {
        let default = Value::Object(Map::new());

        Ok(self
            .epsilon_controller
            .create_epsilon_instance(template_name, content.unwrap_or(default))
            .await?)
    }

    pub async fn remove_instance(&self, name: &str) -> Result<(), EpsilonError> {
        info!("An instance has been removed (name={})", name);

        self.epsilon_controller
            .get_epsilon_instance_api()
            .delete(name, &DeleteParams::default())
            .await
            .map_err(|_| EpsilonError::RemoveInstanceError(name.to_owned()))?;

        Ok(())
    }

    pub async fn get_instance(&self, instance_name: &str) -> Result<EpsilonInstance, EpsilonError> {
        Ok(self
            .epsilon_controller
            .get_epsilon_instance_api()
            .get(instance_name)
            .await
            .map_err(|_| EpsilonError::RetrieveInstanceError)?)
    }

    pub async fn get_instances(
        &self,
        type_option: Option<InstanceType>,
        template_option: Option<&str>,
        state_option: Option<EpsilonState>,
    ) -> Result<Vec<Arc<EpsilonInstance>>, EpsilonError> {
        let instances = self.epsilon_controller.get_epsilon_instance_store().state();

        for instance in &instances {
            instance
                .status
                .as_ref()
                .ok_or(EpsilonError::RetrieveInstanceError)?;
        }

        Ok(instances
            .into_iter()
            .filter(|instance| {
                let status = instance.status.as_ref().unwrap();

                let condition1 = if let Some(template_name) = template_option {
                    instance.spec.template == template_name
                } else {
                    true
                };

                let condition2 = if let Some(instance_type) = type_option {
                    status.t == instance_type
                } else {
                    true
                };

                let condition3 = if let Some(state) = state_option {
                    status.state == state
                } else {
                    true
                };

                condition1 && condition2 && condition3
            })
            .collect())
    }

    pub async fn enable_in_game_instance(&self, name: &str) -> Result<(), EpsilonError> {
        self.epsilon_controller.in_game_epsilon_instance(name).await
    }
}
