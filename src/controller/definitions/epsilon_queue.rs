use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Deserialize, Serialize, Clone, Debug, JsonSchema)]
#[kube(
    group = "controller.epsilon.fr",
    version = "v1",
    kind = "EpsilonQueue",
    printcolumn = r#"{"name":"Target", "type":"string", "description":"Template name target of queue", "jsonPath":".spec.target"}"#,
    namespaced
)]
pub struct EpsilonQueueSpec {
    pub target: String,
}
