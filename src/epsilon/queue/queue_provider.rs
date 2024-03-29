use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::epsilon::epsilon_error::EpsilonError;
use crate::epsilon::queue::common::epsilon_queue::Queue;
use crate::{InstanceProvider, TemplateProvider};

pub struct QueueProvider {
    queue_map: HashMap<String, RwLock<Queue>>,
}

impl QueueProvider {
    pub async fn new(
        _instance_provider: &InstanceProvider,
        template_provider: &Arc<TemplateProvider>,
    ) -> Result<QueueProvider, EpsilonError> {
        let mut map = HashMap::new();

        for template in template_provider.get_templates().await? {
            map.insert(template.name.to_owned(), RwLock::new(Queue::new()));
        }

        Ok(QueueProvider { queue_map: map })
    }

    pub fn get_queues(&self) -> &HashMap<String, RwLock<Queue>> {
        &self.queue_map
    }
}
