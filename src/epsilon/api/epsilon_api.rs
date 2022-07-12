use crate::epsilon::api::common::epsilon_events::EpsilonEvent;
use tokio::sync::broadcast::{channel, Receiver, Sender};

pub struct EpsilonApi {
    channel: Sender<EpsilonEvent>,
}

impl EpsilonApi {
    pub fn new() -> EpsilonApi {
        Self {
            channel: channel::<EpsilonEvent>(1024).0,
        }
    }

    pub fn send(&self, event: EpsilonEvent) {
        self.channel.send(event).unwrap();
    }

    pub fn subscribe(&self) -> Receiver<EpsilonEvent> {
        self.channel.subscribe()
    }
}
