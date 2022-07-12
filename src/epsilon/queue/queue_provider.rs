use crate::epsilon::queue::common::epsilon_queue::Queue;
use crate::{EResult, InstanceProvider};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct QueueProvider {
    queue_map: HashMap<String, RwLock<Queue>>,
}

impl QueueProvider {
    pub async fn new(instance_provider: &InstanceProvider) -> EResult<QueueProvider> {
        let mut map = HashMap::new();

        for template in instance_provider.get_templates().await? {
            map.insert(String::from(&template.name), RwLock::new(Queue::new()));
        }

        Ok(QueueProvider { queue_map: map })
    }

    pub fn get_queues(&self) -> &HashMap<String, RwLock<Queue>> {
        &self.queue_map
    }
}
