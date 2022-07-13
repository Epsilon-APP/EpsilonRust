use crate::epsilon::server::instances::common::instance::Instance;
use crate::epsilon::server::instances::common::instance_type::InstanceType;
use crate::epsilon::server::instances::common::state::EpsilonState;
use crate::epsilon::server::templates::template::Template;
use crate::k8s::label::Label;
use crate::{EResult, Kube, TemplateProvider};
use anyhow::format_err;
use serde_json::json;
use std::env;
use std::sync::Arc;

pub struct InstanceProvider {
    kube: Arc<Kube>,
    template_provider: Arc<TemplateProvider>,
}

impl InstanceProvider {
    pub fn new(kube: &Arc<Kube>, template_provider: &Arc<TemplateProvider>) -> InstanceProvider {
        Self {
            kube: Arc::clone(kube),
            template_provider: Arc::clone(template_provider),
        }
    }

    pub async fn start_instance(&self, template_name: &str) -> EResult<Instance> {
        let template = self.template_provider.get_template(template_name).await?;

        let instance = Instance::new(&self.kube, &template).await?;

        info!(
            "An instance [{}] has been created (template={}, type={})",
            instance.get_name(),
            instance.get_template_name(),
            instance.get_instance_type().to_string()
        );

        Ok(instance)
    }

    pub async fn remove_instance(&self, name: &str) -> EResult<()> {
        info!("An instance has been removed (name={})", name);

        Ok(self.kube.delete_pod(name).await?)
    }

    pub async fn get_instance(&self, instance_name: &str) -> EResult<Instance> {
        let pod = self.kube.get_pod(instance_name).await?;
        let instance = Instance::from_pod(pod);

        Ok(instance)
    }

    pub async fn get_instances(
        &self,
        instance_type: &InstanceType,
        template_option: Option<&str>,
        state_option: Option<&EpsilonState>,
        get_all: bool,
    ) -> EResult<Vec<Instance>> {
        let mut labels = vec![Label::get_instance_type_label(instance_type)];

        if let Some(template_name) = &template_option {
            labels.push(Label::get_template_label(template_name));
        }

        let pods = self.kube.get_pods(Some(&labels), None).await?;

        let map = pods.into_iter().map(Instance::from_pod);

        match state_option {
            Some(state) => Ok(map
                .filter(|instance| instance.get_state() == *state)
                .collect()),

            None => Ok(map
                .filter(|instance| get_all || !instance.is_failed())
                .collect()),
        }
    }

    pub async fn set_in_game_instance(&self, name: &str, enable: bool) -> EResult<()> {
        let label = &Label::get_in_game_label(enable);

        let patch = json!({
            "metadata": {
                "labels": {
                    label.get_key(): label.get_value()
                }
            }
        });

        Ok(self.kube.patch_pod(name, &patch).await?)
    }
}
