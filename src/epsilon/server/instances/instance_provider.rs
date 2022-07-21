use crate::controller::definitions::epsilon_instance::{
    EpsilonInstance, EpsilonInstanceSpec, EpsilonInstanceStatus,
};
use crate::epsilon::server::instances::common::instance_type::InstanceType;
use crate::epsilon::server::instances::common::state::EpsilonState;
use crate::epsilon::server::templates::template::Template;
use crate::k8s::label::Label;
use crate::{EResult, EpsilonController, Kube, TemplateProvider};
use anyhow::format_err;
use futures::StreamExt;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{DeleteParams, ListParams, PostParams};
use kube::Api;
use serde_json::json;
use std::env;
use std::sync::Arc;

pub struct InstanceProvider {
    epsilon_controller: Arc<EpsilonController>,
    template_provider: Arc<TemplateProvider>,
}

impl InstanceProvider {
    pub fn new(
        epsilon_controller: &Arc<EpsilonController>,
        template_provider: &Arc<TemplateProvider>,
    ) -> InstanceProvider {
        Self {
            epsilon_controller: Arc::clone(epsilon_controller),
            template_provider: Arc::clone(template_provider),
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
        get_all: bool,
    ) -> EResult<Vec<Arc<EpsilonInstance>>> {
        let mut instances = self.epsilon_controller.get_epsilon_instance_store().state();

        instances = instances
            .into_iter()
            .filter(|instance| {
                instance.status.is_none() || instance.status.as_ref().unwrap().t == *instance_type
            })
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
                .filter(|instance| {
                    instance.status.is_none() || instance.status.as_ref().unwrap().state == *state
                })
                .collect();
        };

        Ok(instances)
    }

    pub async fn set_in_game_instance(&self, name: &str, enable: bool) -> EResult<()> {
        Ok(())
    }
}
