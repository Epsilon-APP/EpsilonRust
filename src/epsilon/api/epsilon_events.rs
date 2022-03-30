use crate::epsilon::queue::queue_provider::Group;

#[derive(Clone)]
pub enum EpsilonEvent {
    SendToServer(Group, String),
    ClearServer(String),
}

impl ToString for EpsilonEvent {
    fn to_string(&self) -> String {
        String::from(match self {
            EpsilonEvent::SendToServer(_, _) => "SendToServer",
            EpsilonEvent::ClearServer(_) => "ClearServer",
        })
    }
}
