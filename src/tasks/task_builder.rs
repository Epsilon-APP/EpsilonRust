use std::sync::Arc;

use crate::{InstanceProvider, Task};
use std::time::Duration;

pub struct TaskBuilder {}

impl TaskBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn ignite_task(&self, mut task: Box<dyn Task>, millis: u64) -> &Self {
        let task_name = task.get_name();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(millis));
            loop {
                interval.tick().await;

                if let Err(err) = task.run().await {
                    error!("{}", err);
                }
            }
        });

        info!("Task {} ignited", task_name);

        self
    }
}
