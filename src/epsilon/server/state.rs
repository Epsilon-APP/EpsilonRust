use serde::Serialize;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum EpsilonState {
    Starting,
    Running,
    InGame,
    Stopping,
}
