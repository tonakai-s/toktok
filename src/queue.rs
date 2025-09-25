use std::{cmp::Reverse, collections::BinaryHeap};

use crate::task::Task;

#[derive(Debug)]
pub struct PriorityQueue {
    heap: BinaryHeap<Reverse<Task>>,
}

impl PriorityQueue {
    pub fn default() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub fn dequeue(&mut self) -> Task {
        self.heap.pop().unwrap().0
    }

    pub fn enqueue(&mut self, item: Task) {
        self.heap.push(Reverse(item));
    }

    pub fn peek(&self) -> Option<&Reverse<Task>> {
        self.heap.peek()
    }
}
