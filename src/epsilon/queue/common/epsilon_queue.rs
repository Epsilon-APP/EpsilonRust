use std::collections::{HashSet, VecDeque};

use crate::epsilon::queue::common::group::Group;

pub struct Queue {
    queue: VecDeque<Group>,
    in_queue: HashSet<String>,
}

impl Queue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            in_queue: HashSet::new(),
        }
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
