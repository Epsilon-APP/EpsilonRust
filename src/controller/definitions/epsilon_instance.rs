use async_trait::async_trait;
use std::sync::Arc;

use crate::epsilon::server::instances::common::instance_type::InstanceType;
use crate::epsilon::server::instances::common::state::EpsilonState;
use crate::EResult;

use async_minecraft_ping::{ConnectionConfig, StatusResponse};
use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;

#[derive(CustomResource, Debug, Serialize, Deserialize, Default, Clone, JsonSchema)]
#[kube(
    group = "controller.epsilon.fr",
    version = "v1",
    kind = "EpsilonInstance",
    status = "EpsilonInstanceStatus",
    printcolumn = r#"{"name":"Template", "type":"string", "description":"Template name of instance", "jsonPath":".spec.template"}"#,
    printcolumn = r#"{"name":"State", "type":"string", "description":"State of instance", "jsonPath":".status.state"}"#,
    namespaced
)]
pub struct EpsilonInstanceSpec {
    pub template: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct EpsilonInstanceStatus {
    pub ip: Option<String>,

    pub template: String,
    pub t: InstanceType,

    pub hub: bool,

    pub content: String,

    pub state: EpsilonState,

    pub slots: i32,

    pub close: bool,
}

impl EpsilonInstance {
    pub async fn to_json(&self) -> InstanceJson {
        InstanceJson {
            name: self.metadata.name.as_ref().unwrap().clone(),
            template: self.spec.template.clone(),
            state: self.get_state().clone(),
            slots: self.status.as_ref().unwrap().slots,
            online_count: self.get_online_count().await.unwrap_or(0),
        }
    }

    pub fn get_state(&self) -> &EpsilonState {
        match &self.status {
            None => &EpsilonState::Starting,
            Some(status) => &status.state,
        }
    }

    pub async fn get_info(&self) -> EResult<StatusResponse> {
        let status = self.status.as_ref().unwrap();
        let address = status.ip.as_ref().unwrap();
        let config = ConnectionConfig::build(address);

        let duration = Duration::from_millis(150);

        Ok(timeout(duration, async move {
            config
                .connect()
                .await
                .unwrap()
                .status()
                .await
                .unwrap()
                .status
        })
        .await?)
    }

    pub async fn get_online_count(&self) -> EResult<i32> {
        Ok(self.get_info().await?.players.online as i32)
    }

    pub async fn get_available_slots(&self) -> EResult<i32> {
        Ok(&self.status.as_ref().unwrap().slots - self.get_online_count().await?)
    }
}

#[async_trait]
pub trait VectorOfInstance {
    async fn get_available_slots(&self) -> EResult<i32>;
    async fn get_online_count(&self) -> EResult<i32>;
}

#[async_trait]
impl VectorOfInstance for Vec<Arc<EpsilonInstance>> {
    async fn get_available_slots(&self) -> EResult<i32> {
        let mut available_slots = 0;

        for instance in self {
            available_slots += instance.get_available_slots().await?;
        }

        Ok(available_slots)
    }

    async fn get_online_count(&self) -> EResult<i32> {
        let mut number = 0;

        for instance in self {
            number += instance.get_online_count().await?;
        }

        Ok(number)
    }
}

#[derive(Serialize)]
pub struct InstanceJson {
    pub name: String,
    pub template: String,
    pub state: EpsilonState,
    pub slots: i32,
    pub online_count: i32,
}
