use crate::epsilon::queue::common::group::Group;

#[derive(Debug, Clone)]
pub enum EpsilonEvent {
    SendToServer(Group, String),
}

impl ToString for EpsilonEvent {
    fn to_string(&self) -> String {
        match self {
            EpsilonEvent::SendToServer(_, _) => "SendToServer",
        }
        .to_owned()
    }
}
