use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum EpsilonState {
    Starting,
    Running,
    InGame,
    Stopping,
}
