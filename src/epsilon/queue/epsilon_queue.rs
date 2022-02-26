use crate::epsilon::queue::queue_provider::Group;
use crate::epsilon::server::template::Template;
use std::collections::VecDeque;

pub struct Queue {
    template: Template,
    queue: VecDeque<Group>,
}

impl Queue {
    pub fn new(template: Template) -> Self {
        Self {
            template,
            queue: VecDeque::new(),
        }
    }

    pub fn get_template(&self) -> &Template {
        &self.template
    }

    pub fn push(&mut self, group: Group) {
        self.queue.push_back(group);
    }

    pub fn pop(&mut self) -> Option<Group> {
        self.queue.pop_front()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
