use std::sync::Arc;

use rocket::{Shutdown, State};
use rocket::response::stream::{Event, EventStream};
use serde_json::json;
use tokio::select;
use tokio::sync::broadcast::error::RecvError;

use crate::Context;
use crate::epsilon::api::common::epsilon_events::EpsilonEvent;

#[rocket::get("/ping")]
pub async fn ping() -> &'static str {
    "Pong"
}

#[rocket::get("/events")]
pub async fn events(context: &State<Arc<Context>>, mut end: Shutdown) -> EventStream![] {
    let epsilon_api = context.get_epsilon_api();

    let mut rx = epsilon_api.subscribe();

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
