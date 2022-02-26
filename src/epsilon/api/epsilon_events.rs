use crate::epsilon::queue::queue_provider::Group;

#[derive(Clone)]
pub enum EpsilonEvent {
    SendToServer(Group, String),
}
