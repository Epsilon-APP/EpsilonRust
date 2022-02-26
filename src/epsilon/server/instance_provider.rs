use crate::k8s::label::Label;
use crate::{EResult, Kube};
use std::collections::HashMap;

use crate::epsilon::server::instance::Instance;
use crate::epsilon::server::instance_type::InstanceType;
use crate::epsilon::server::template::Template;
use anyhow::format_err;
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
        let template = self.get_template(template_name).await?;
        let instance_type = template.t.parse()?;

        let instance = Instance::new(&self.kube, template_name, &instance_type).await?;

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
    ) -> EResult<Vec<Instance>> {
        let mut labels = vec![Label::get_instance_type_label(instance_type)];

        if let Some(template_name) = &template_option {
            labels.push(Label::get_template_label(template_name));
        }

        let pods = self.kube.get_pods(Some(&labels), None).await?;

        Ok(pods.into_iter().map(Instance::from_pod).collect())
    }

    pub async fn get_ready_instances(
        &self,
        instance_type: &InstanceType,
        template_option: Option<&str>,
    ) -> EResult<Vec<Instance>> {
        let instances = self.get_instances(instance_type, template_option).await?;

        Ok(instances
            .into_iter()
            .filter(|instance| instance.is_ready())
            .collect())
    }

    pub async fn get_template_host(&self, route: &str) -> String {
        let labels = vec![Label::new("epsilon.fr/template-provider", "true")];

        let pods_result = self.kube.get_pods(Some(&labels), None).await;

        if let Ok(pods) = pods_result {
            let pod = pods.first();

            if let Some(pod) = pod {
                let status = pod.status.as_ref().unwrap();
                let pod_ip = status.pod_ip.as_ref().unwrap();

                return format!("http://{}:3333/{}", pod_ip, route);
            }
        }

        format!("http://127.0.0.1:3333/{}", route)
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
