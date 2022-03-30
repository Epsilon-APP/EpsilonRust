use serde::Serialize;

#[derive(Serialize, PartialEq, Eq)]
pub enum EpsilonState {
    Starting,
    Running,
    InGame,
    Stopping,
}
