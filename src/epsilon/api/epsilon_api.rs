use crate::epsilon::api::epsilon_events::EpsilonEvent;
use rocket::response::stream::{Event, EventStream};
use rocket::{Shutdown, State};
use serde_json::json;
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::{channel, Sender};

pub struct EpsilonApi {
    channel: Sender<EpsilonEvent>,
}

impl EpsilonApi {
    pub fn new() -> Arc<EpsilonApi> {
        Arc::new(Self {
            channel: channel::<EpsilonEvent>(1024).0,
        })
    }

    pub fn send(&self, event: EpsilonEvent) {
        self.channel.send(event);
    }
}

#[rocket::get("/ping")]
pub async fn ping() -> &'static str {
    "Pong"
}

#[rocket::get("/events")]
pub async fn events(epsilon_api: &State<Arc<EpsilonApi>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = epsilon_api.channel.subscribe();

    EventStream! {
        loop {
            let event: EpsilonEvent = select! {
                event = rx.recv() => match event {
                    Ok(event) => event,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            match &event {
                EpsilonEvent::SendToServer(group, server) => {
                    info!("Send to server {:?} [{}]", group, server);

                    let json = json!({
                        "group": group,
                        "server": server,
                    });

                    yield Event::data(json.to_string()).event(event.to_string());
                }
            }
        }
    }
}
