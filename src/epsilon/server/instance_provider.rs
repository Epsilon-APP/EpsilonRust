use crate::k8s::label::Label;
use crate::{EResult, Kube};
use std::env;

use crate::epsilon::epsilon_error::EpsilonError;
use crate::epsilon::server::instance::{Instance, InstanceJson};
use crate::epsilon::server::instance_type::InstanceType;
use crate::epsilon::server::state::EpsilonState;
use crate::epsilon::server::templates::template::Template;
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

    pub async fn get_template_host(&self, route: &str) -> String {
        format!(
            "http://{}:8000/{}",
            env::var("HOST_TEMPLATE").unwrap(),
            route
        )
    }

    pub async fn get_template(&self, template_name: &str) -> EResult<Template> {
        let url = self
            .get_template_host(&format!("templates/{}", template_name))
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

    pub async fn get_templates(&self) -> EResult<Vec<Template>> {
        let url = self.get_template_host("templates").await;

        debug!("Fetching template list from {}", url);

        let request = reqwest::get(&url).await?;

        if request.status().is_success() {
            let template = request.json::<Vec<Template>>().await?;

            Ok(template)
        } else {
            Err(format_err!("Failed to fetch template list from {}", url))
        }
    }
}

#[rocket::post("/create/<template>")]
pub async fn create(template: &str, instance_provider: &State<Arc<InstanceProvider>>) {
    instance_provider
        .start_instance(template)
        .await
        .map_err(|_e| {
            EpsilonError::ApiServerError(format!(
                "Failed to create an instance from template ({})",
                template
            ))
        })
        .unwrap();

    info!("An instance has been created (template={})", template);
}

#[rocket::post("/close/<instance>")]
pub async fn close(instance: &str, instance_provider: &State<Arc<InstanceProvider>>) {
    instance_provider
        .remove_instance(instance)
        .await
        .map_err(|_| {
            EpsilonError::ApiServerError(format!("Failed to close instance ({})", instance))
        })
        .unwrap();
}

#[rocket::post("/in_game/<instance>")]
pub async fn in_game(instance: &str, instance_provider: &State<Arc<InstanceProvider>>) {
    instance_provider
        .set_in_game_instance(instance, true)
        .await
        .map_err(|_| {
            EpsilonError::ApiServerError(format!("Failed to set in game instance ({})", instance))
        })
        .unwrap();

    info!("An instance is now in game (name={})", instance);
}

#[rocket::get("/get/<template>")]
pub async fn get(template: &str, instance_provider: &State<Arc<InstanceProvider>>) -> String {
    let instances = instance_provider
        .get_instances(&InstanceType::Server, Some(template), None, false)
        .await
        .map_err(|_| {
            EpsilonError::ApiServerError(format!(
                "Failed to get instance from template {}",
                template
            ))
        })
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
    info!("Fetching all instances");

    let instances = instance_provider
        .get_instances(&InstanceType::Server, None, None, false)
        .await
        .map_err(|_| EpsilonError::ApiServerError("Failed to get every instance".to_string()))
        .unwrap()
        .into_iter();

    info!("Fetched {} instances", instances.len());

    let mut json_array: Vec<InstanceJson> = Vec::new();

    for instance in instances {
        let json = instance.to_json().await;

        info!("JSON");

        json_array.push(json);

        info!("PUSH")
    }

    info!("Converted {} instances to json", json_array.len());

    json!({ "instances": json_array }).to_string()
}

#[rocket::get("/get_from_name/<instance_name>")]
pub async fn get_from_name(
    instance_name: &str,
    instance_provider: &State<Arc<InstanceProvider>>,
) -> String {
    let instance = instance_provider.get_instance(instance_name).await.unwrap();

    serde_json::to_string(&instance.to_json().await).unwrap()
}
