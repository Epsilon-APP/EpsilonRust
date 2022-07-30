use tokio::sync::broadcast::{channel, Receiver, Sender};

use crate::epsilon::api::common::epsilon_events::EpsilonEvent;
use crate::epsilon::epsilon_error::EpsilonError;

pub struct EpsilonApi {
    channel: Sender<EpsilonEvent>,
}

impl EpsilonApi {
    pub fn new() -> EpsilonApi {
        Self {
            channel: channel::<EpsilonEvent>(1024).0,
        }
    }

    pub fn send(&self, event: EpsilonEvent) -> Result<(), EpsilonError> {
        let event_name = event.to_string();

        self.channel
            .send(event)
            .map_err(|_| EpsilonError::SendEventError(event_name))?;

        Ok(())
    }

    pub fn subscribe(&self) -> Receiver<EpsilonEvent> {
        self.channel.subscribe()
    }
}
