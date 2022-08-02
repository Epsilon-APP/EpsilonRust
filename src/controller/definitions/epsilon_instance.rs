use async_trait::async_trait;
use std::sync::Arc;

use crate::epsilon::server::instances::common::instance_type::InstanceType;
use crate::epsilon::server::instances::common::state::EpsilonState;

use crate::epsilon::epsilon_error::EpsilonError;
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
    printcolumn = r#"{"name":"Online", "type":"integer", "description":"Online count of instance", "jsonPath":".status.online"}"#,
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
    pub online: i32,

    pub close: bool,
}

impl EpsilonInstance {
    pub async fn to_json(&self) -> Result<InstanceJson, EpsilonError> {
        let status = self
            .status
            .as_ref()
            .ok_or(EpsilonError::RetrieveStatusError)?
            .clone();

        Ok(InstanceJson {
            name: self.get_name(),
            template: self.spec.template.clone(),

            content: status.content,

            hub: status.hub,

            t: status.t,
            state: self.get_state(),

            slots: status.slots,
            online_count: self.get_online_count().await.unwrap_or(0),

            ip: status.ip,
        })
    }

    pub fn get_name(&self) -> String {
        self.metadata.name.as_ref().unwrap().to_owned()
    }

    pub fn get_state(&self) -> EpsilonState {
        match &self.status {
            None => EpsilonState::Starting,
            Some(status) => status.state,
        }
    }

    pub async fn get_info(&self) -> Result<StatusResponse, EpsilonError> {
        let status = self
            .status
            .as_ref()
            .ok_or(EpsilonError::RetrieveStatusError)?;

        let address = status
            .ip
            .as_ref()
            .ok_or(EpsilonError::RetrieveIpAddressError)?;

        let port = status.t.get_entry_port();

        let mut config = ConnectionConfig::build(address);
        config = config.with_port(port as u16);

        let duration = Duration::from_millis(150);

        let status_result: Result<StatusResponse, EpsilonError> = timeout(duration, async move {
            Ok(config.connect().await?.status().await?.status)
        })
        .await?;

        status_result
    }

    pub async fn get_online_count(&self) -> Result<i32, EpsilonError> {
        Ok(self.get_info().await?.players.online as i32)
    }

    pub async fn get_available_slots(&self) -> Result<i32, EpsilonError> {
        Ok(&self
            .status
            .as_ref()
            .ok_or(EpsilonError::RetrieveStatusError)?
            .slots
            - self.get_online_count().await?)
    }
}

#[async_trait]
pub trait VectorOfInstance {
    async fn get_available_slots(&self) -> Result<i32, EpsilonError>;
    async fn get_online_count(&self) -> Result<i32, EpsilonError>;
}

#[async_trait]
impl VectorOfInstance for Vec<Arc<EpsilonInstance>> {
    async fn get_available_slots(&self) -> Result<i32, EpsilonError> {
        let mut available_slots = 0;

        for instance in self {
            available_slots += instance.get_available_slots().await?;
        }

        Ok(available_slots)
    }

    async fn get_online_count(&self) -> Result<i32, EpsilonError> {
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

    pub content: String,

    pub hub: bool,

    #[serde(rename = "type")]
    pub t: InstanceType,
    pub state: EpsilonState,

    pub slots: i32,
    pub online_count: i32,

    pub ip: Option<String>,
}
