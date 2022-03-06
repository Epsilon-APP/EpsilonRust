use serde::Serialize;

#[derive(Serialize, PartialEq)]
pub enum EpsilonState {
    Starting,
    Running,
    InGame,
    Stopping,
}
