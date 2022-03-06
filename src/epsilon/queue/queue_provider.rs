use crate::epsilon::api::epsilon_events::EpsilonEvent;
use crate::epsilon::queue::epsilon_queue::Queue;
use crate::{EResult, InstanceProvider};
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct QueueProvider {
    queue_map: HashMap<String, Queue>,
}

impl QueueProvider {
    pub async fn new(
        instance_provider: &Arc<InstanceProvider>,
    ) -> EResult<Arc<Mutex<QueueProvider>>> {
        let mut map = HashMap::new();

        for (template_name, template) in instance_provider.get_templates().await? {
            map.insert(template_name, Queue::new(template));
        }

        Ok(Arc::new(Mutex::new(QueueProvider { queue_map: map })))
    }

    pub fn get_queues(&mut self) -> &mut HashMap<String, Queue> {
        &mut self.queue_map
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub players: Vec<String>,
    pub queue: String,
}

#[rocket::post("/push", data = "<body>")]
pub async fn push(body: Json<Group>, queue_provider: &State<Arc<Mutex<QueueProvider>>>) {
    let mut queue_provider = queue_provider.lock().await;

    let queue_map = queue_provider.get_queues();
    let queue = queue_map.get_mut(&body.queue).unwrap();

    info!(
        "Player {} added to queue {}",
        &body.players.join("/"),
        &body.queue
    );

    queue.push(body.0);
}
