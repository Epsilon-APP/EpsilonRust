use async_trait::async_trait;

use crate::epsilon::server::instance_type::InstanceType;
use crate::k8s::label::Label;
use crate::{EResult, Kube};
use async_minecraft_ping::ServerDescription::Plain;
use async_minecraft_ping::{
    ConnectionConfig, ServerDescription, ServerError, ServerPlayers, ServerVersion,
    StatusConnection, StatusResponse,
};
use k8s_openapi::api::core::v1::Pod;
use std::sync::Arc;

pub struct Instance {
    pod: Pod,
}

impl Instance {
    pub async fn new(
        kube: &Arc<Kube>,
        template_name: &str,
        instance_type: &InstanceType,
    ) -> EResult<Self> {
        let instance_label = Label::get_instance_type_label(instance_type);
        let template_label = Label::get_template_label(template_name);

        let port = instance_type.get_associated_port();

        let labels = vec![instance_label, template_label];

        let pod = kube
            .create_epsilon_pod(template_name, Some(&labels), port)
            .await?;

        Ok(Self { pod })
    }

    pub fn from_pod(pod: Pod) -> Self {
        let instance = Self { pod };

        assert!(instance.is_valid());

        instance
    }

    pub fn get_name(&self) -> &str {
        self.pod.metadata.name.as_ref().unwrap()
    }

    pub fn get_template_name(&self) -> &str {
        self.pod
            .metadata
            .labels
            .as_ref()
            .unwrap()
            .get("epsilon.fr/template")
            .unwrap()
    }

    pub fn get_instance_type(&self) -> InstanceType {
        self.pod
            .metadata
            .labels
            .as_ref()
            .unwrap()
            .get("epsilon.fr/instance")
            .unwrap()
            .parse()
            .unwrap()
    }

    pub async fn get_info(&self) -> EResult<StatusResponse> {
        let status = self.pod.status.as_ref().unwrap();
        let address = status.pod_ip.as_ref().unwrap();

        let config = ConnectionConfig::build(address);

        match config.connect().await {
            Ok(connection) => Ok(connection.status().await?.status),
            Err(_) => Ok(StatusResponse {
                version: ServerVersion {
                    name: "".to_string(),
                    protocol: 0,
                },
                players: ServerPlayers {
                    max: 10,
                    online: 0,
                    sample: None,
                },
                description: Plain("Description".to_string()),
                favicon: None,
            }),
        }
    }

    pub fn is_ready(&self) -> bool {
        let status = self.pod.status.as_ref().unwrap();

        let conditions = status.conditions.as_ref().unwrap();

        conditions
            .iter()
            .any(|condition| condition.type_ == "Ready" && condition.status == "True")
            && status.phase.as_ref().unwrap() == "Running"
    }

    pub fn is_valid(&self) -> bool {
        self.pod
            .metadata
            .labels
            .as_ref()
            .unwrap()
            .contains_key("epsilon.fr/main")
    }
}

#[async_trait]
pub trait VectorOfInstance {
    async fn get_online_count(&self) -> EResult<u32>;
}

#[async_trait]
impl VectorOfInstance for Vec<Instance> {
    async fn get_online_count(&self) -> EResult<u32> {
        let mut number = 0;

        for instance in self {
            number += instance.get_info().await?.players.online;
        }

        Ok(number)
    }
}
