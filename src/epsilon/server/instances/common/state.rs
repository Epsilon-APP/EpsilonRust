use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
pub enum EpsilonState {
    Starting,
    Running,
    InGame,
    Stopping,
}
