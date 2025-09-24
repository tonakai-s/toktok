use std::collections::BinaryHeap;

use crate::task::Task;

pub struct PriorityQueue {
    heap: BinaryHeap<Task>,
}

impl PriorityQueue {
    pub fn default() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub fn dequeue(&mut self) -> Task {
        self.heap.pop().unwrap()
    }

    pub fn enqueue(&mut self, item: Task) {
        self.heap.push(item);
    }

    pub fn peek(&self) -> Option<&Task> {
        self.heap.peek()
    }
}
