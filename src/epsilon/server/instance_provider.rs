use crate::k8s::label::Label;
use crate::{EResult, Kube};
use std::collections::HashMap;

use crate::epsilon::server::instance::{Instance, InstanceJson};
use crate::epsilon::server::instance_type::InstanceType;
use crate::epsilon::server::state::EpsilonState;
use crate::epsilon::server::template::Template;
use anyhow::format_err;
use rocket::State;
use serde_json::json;
use std::sync::Arc;

pub struct InstanceProvider {
    kube: Arc<Kube>,
}

impl InstanceProvider {
    pub fn new(kube: &Arc<Kube>) -> Arc<InstanceProvider> {
        Arc::new(Self {
            kube: Arc::clone(kube),
        })
    }

    pub async fn start_instance(&self, template_name: &str) -> EResult<Instance> {
        let template = &self.get_template(template_name).await?;

        let instance = Instance::new(&self.kube, template).await?;

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

    pub async fn get_instances(
        &self,
        instance_type: &InstanceType,
        template_option: Option<&str>,
        state: Option<&EpsilonState>,
    ) -> EResult<Vec<Instance>> {
        let mut labels = vec![Label::get_instance_type_label(instance_type)];

        if let Some(template_name) = &template_option {
            labels.push(Label::get_template_label(template_name));
        }

        let pods = self.kube.get_pods(Some(&labels), None).await?;

        let map = pods.into_iter().map(Instance::from_pod);

        match state {
            Some(_) => Ok(map
                .filter(|instance| instance.get_state().eq(state.unwrap()))
                .collect()),

            None => Ok(map.collect()),
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

    pub async fn get_template_host(&self, route: &str) -> String {
        format!(
            "http://epsilon-template.epsilon.svc.cluster.local:8000/{}",
            route
        )
    }

    pub async fn get_template(&self, template_name: &str) -> EResult<Template> {
        let url = self
            .get_template_host(&format!("template/{}", template_name))
            .await;

        debug!("Fetching template from {}", url);

        let request = reqwest::get(&url).await?;

        if request.status().is_success() {
            let template = request.json::<Template>().await?;

            Ok(template)
        } else {
            Err(format_err!("Failed to fetch template from {}", url))
        }
    }

    pub async fn get_templates(&self) -> EResult<HashMap<String, Template>> {
        let url = self.get_template_host("templates").await;

        debug!("Fetching template list from {}", url);

        let request = reqwest::get(&url).await?;

        if request.status().is_success() {
            let template = request.json::<HashMap<String, Template>>().await?;

            Ok(template)
        } else {
            Err(format_err!("Failed to fetch template list from {}", url))
        }
    }
}

#[rocket::post("/create/<template_name>")]
pub async fn create(template_name: &str, instance_provider: &State<Arc<InstanceProvider>>) {
    instance_provider
        .start_instance(template_name)
        .await
        .unwrap();

    info!("An instance has been created (template={})", template_name);
}

#[rocket::post("/close/<instance_name>")]
pub async fn close(instance_name: &str, instance_provider: &State<Arc<InstanceProvider>>) {
    instance_provider
        .remove_instance(instance_name)
        .await
        .unwrap();
}

#[rocket::post("/in_game/<instance>")]
pub async fn in_game(instance: &str, instance_provider: &State<Arc<InstanceProvider>>) {
    instance_provider
        .set_in_game_instance(instance, true)
        .await
        .unwrap();

    info!("An instance is now in game (name={})", instance);
}

#[rocket::get("/get/<template_name>")]
pub async fn get(template_name: &str, instance_provider: &State<Arc<InstanceProvider>>) -> String {
    let instances = instance_provider
        .get_instances(&InstanceType::Server, Some(template_name), None)
        .await
        .unwrap()
        .into_iter();

    let mut json_array: Vec<InstanceJson> = Vec::with_capacity(instances.len());

    for instance in instances {
        json_array.push(instance.to_json().await);
    }

    json!({ "instances": json_array }).to_string()
}

#[rocket::get("/get_all")]
pub async fn get_all(instance_provider: &State<Arc<InstanceProvider>>) -> String {
    let instances = instance_provider
        .get_instances(&InstanceType::Server, None, None)
        .await
        .unwrap()
        .into_iter();

    let mut json_array: Vec<InstanceJson> = Vec::with_capacity(instances.len());

    for instance in instances {
        json_array.push(instance.to_json().await);
    }

    json!({ "instances": json_array }).to_string()
}
