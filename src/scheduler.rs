use std::{
    sync::{Arc, Mutex, mpsc::channel},
    thread,
    time::Duration,
};

use jiff::Zoned;

use crate::{executor, queue::PriorityQueue, task::Task};

pub struct Scheduler {
    tasks: Arc<Mutex<PriorityQueue>>,
}
impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(PriorityQueue::default())),
        }
    }
    pub fn enqueue(&mut self, task: Task) {
        self.tasks.lock().unwrap().enqueue(task);
    }
    pub async fn start(&self) {
        let (tx, rx) = channel::<Task>();
        let task_queue = self.tasks.clone();
        tokio::spawn(async move {
            loop {
                let mut task = rx.recv().unwrap();
                task.set_next_execution_at();
                task_queue.lock().unwrap().enqueue(task);
            }
        });

        loop {
            thread::sleep(Duration::from_secs(1));
            {
                let mut task_queue = self.tasks.lock().unwrap();
                loop {
                    if task_queue
                        .peek()
                        .is_none_or(|t| t.next_execution_at() > &Zoned::now().datetime())
                    {
                        break;
                    }

                    let task = task_queue.dequeue();
                    dbg!(&task);
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        println!("--Lets spawn a task for '{}'!", task.name());
                        executor::execute(task, tx).await;
                    });
                }
            }
        }
    }
}
