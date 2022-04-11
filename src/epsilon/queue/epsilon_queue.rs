use crate::epsilon::queue::queue_provider::Group;
use crate::epsilon::server::templates::template::Template;

use std::collections::{HashSet, VecDeque};

pub struct Queue {
    template: Template,
    queue: VecDeque<Group>,
    in_queue: HashSet<String>,
}

impl Queue {
    pub fn new(template: Template) -> Self {
        Self {
            template,
            queue: VecDeque::new(),
            in_queue: HashSet::new(),
        }
    }

    pub fn get_template(&self) -> &Template {
        &self.template
    }

    pub fn push(&mut self, group: Group) {
        for player in &group.players {
            if self.in_queue.contains(player) {
                self.queue
                    .retain(|queue_group| !queue_group.players.contains(player));
            }

            self.in_queue.insert(player.into());
        }

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
