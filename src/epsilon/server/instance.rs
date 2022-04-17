use async_trait::async_trait;

use crate::epsilon::server::instance_type::InstanceType;
use crate::k8s::label::Label;
use crate::{EResult, Kube};
use anyhow::format_err;
use async_minecraft_ping::{ConnectionConfig, StatusResponse};
use k8s_openapi::api::core::v1::Pod;
use std::sync::Arc;
use std::time::Duration;

use crate::epsilon::server::state::EpsilonState;
use crate::epsilon::server::templates::template::Template;
use serde::Serialize;

pub struct Instance {
    pod: Pod,
}

#[derive(Serialize)]
pub struct InstanceJson {
    pub name: String,
    pub template: String,
    pub state: EpsilonState,
    pub slots: i32,
    pub online_count: u32,
}

impl Instance {
    pub async fn new(kube: &Arc<Kube>, template: &Template) -> EResult<Self> {
        let template_name = &template.name;
        let instance_type = &template.t;
        let resources = &template.resources;
        let slots = template.slots;

        let instance_label = Label::get_instance_type_label(instance_type);
        let template_label = Label::get_template_label(template_name);
        let slots_label = Label::get_slots_label(slots);

        let labels = vec![instance_label, template_label, slots_label];

        let port = instance_type.get_associated_port();

        let pod = kube
            .create_epsilon_pod(template_name, Some(&labels), port, resources)
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
            .get(Label::TEMPLATE_LABEL)
            .unwrap()
    }

    pub fn get_instance_type(&self) -> InstanceType {
        self.pod
            .metadata
            .labels
            .as_ref()
            .unwrap()
            .get(Label::INSTANCE_TYPE_LABEL)
            .unwrap()
            .parse()
            .unwrap()
    }

    pub fn get_instance_slots(&self) -> i32 {
        self.pod
            .metadata
            .labels
            .as_ref()
            .unwrap()
            .get(Label::SLOTS_LABEL)
            .unwrap()
            .parse()
            .unwrap()
    }

    pub async fn get_info(&self) -> EResult<StatusResponse> {
        let status = self.pod.status.as_ref().unwrap();
        let address_option = status.pod_ip.as_ref();

        if let Some(address) = address_option {
            let timeout = Duration::from_millis(150);
            let config = ConnectionConfig::build(address).with_timeout(timeout);

            Ok(config.connect().await?.status().await?.status)
        } else {
            Err(format_err!("No address found"))
        }
    }

    pub async fn get_online_count(&self) -> EResult<i32> {
        Ok(self.get_info().await?.players.online as i32)
    }

    pub async fn get_available_slots(&self) -> EResult<i32> {
        Ok(self.get_instance_slots() - self.get_online_count().await?)
    }

    pub fn is_succeeded(&self) -> bool {
        self.pod.status.as_ref().unwrap().phase.as_ref().unwrap() == "Succeeded"
    }

    pub fn is_failed(&self) -> bool {
        let phase = self.pod.status.as_ref().unwrap().phase.as_ref().unwrap();

        phase == "Failed" || phase == "Unknown"
    }

    pub fn get_state(&self) -> EpsilonState {
        let status = self.pod.status.as_ref().unwrap();

        let metadata = &self.pod.metadata;
        let labels = metadata.labels.as_ref().unwrap();

        let conditions = status.conditions.as_ref().unwrap();

        let is_ready = conditions
            .iter()
            .any(|condition| condition.type_ == "Ready" && condition.status == "True")
            && status.phase.as_ref().unwrap() == "Running";

        let label = &labels.get(Label::IN_GAME_LABEL);
        let is_in_game = label.is_some() && label.unwrap() == "true";

        let is_stopping = metadata.deletion_timestamp.is_some() || self.is_succeeded();

        if is_ready && is_in_game {
            EpsilonState::InGame
        } else if is_ready {
            EpsilonState::Running
        } else if is_stopping {
            EpsilonState::Stopping
        } else {
            EpsilonState::Starting
        }
    }

    pub fn is_valid(&self) -> bool {
        self.pod
            .metadata
            .labels
            .as_ref()
            .unwrap()
            .contains_key(Label::DEFAULT_LABEL)
    }

    pub async fn to_json(&self) -> InstanceJson {
        let info_result = self.get_info().await;

        InstanceJson {
            name: self.get_name().to_string(),
            template: self.get_template_name().to_string(),
            state: self.get_state(),
            slots: self.get_instance_slots(),
            online_count: match info_result {
                Ok(info) => info.players.online,
                Err(_) => 0,
            },
        }
    }
}

#[async_trait]
pub trait VectorOfInstance {
    async fn get_available_slots(&self) -> EResult<i32>;
    async fn get_online_count(&self) -> i32;
}

#[async_trait]
impl VectorOfInstance for Vec<Instance> {
    async fn get_available_slots(&self) -> EResult<i32> {
        if self.is_empty() {
            return Err(format_err!("No instances found"));
        }

        let mut available_slots = 0;

        for instance in self {
            available_slots += instance.get_available_slots().await?;
        }

        Ok(available_slots)
    }

    async fn get_online_count(&self) -> i32 {
        let mut number = 0;

        for instance in self {
            number += instance.get_online_count().await.unwrap_or(0)
        }

        number
    }
}
