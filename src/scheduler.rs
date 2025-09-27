use std::{
    sync::{Arc, Mutex, mpsc::channel},
    thread,
    time::Duration,
};

use jiff::Zoned;
use tracing::{Level, event};

use crate::{
    checker::structs::CheckerResult, configuration::Configuration, executor, notification::Notifier, queue::PriorityQueue, task::Task
};

#[derive(Debug)]
pub struct Scheduler {
    tasks: Arc<Mutex<PriorityQueue>>,
}

impl Scheduler {
    pub fn new(config: Configuration) -> Self {
        let queue = Arc::new(Mutex::new(PriorityQueue::default()));
        {
            let mut lock = queue.lock().unwrap();
            for task in config.tasks {
                lock.enqueue(task);
            }
        }

        Self { tasks: queue }
    }

    pub async fn start(&self, notifiers: Vec<impl Notifier + Send + 'static>) {
        event!(
            Level::INFO,
            notifiers_count = notifiers.len(),
            "Scheduler has been initiated"
        );

        let (tx_task, rx_task) = channel::<Task>();
        let (tx_notifier, rx_notifier) = channel::<CheckerResult>();
        let task_queue = self.tasks.clone();
        tokio::spawn(async move {
            event!(
                Level::INFO,
                "Task update and enqueuer thread has been initiated"
            );
            loop {
                let mut task = rx_task.recv().unwrap();
                task.set_next_execution_at();
                task_queue.lock().unwrap().enqueue(task);
            }
        });

        tokio::spawn(async move {
            event!(Level::INFO, "Notifiers thread has been initiated");
            loop {
                let execution_result = rx_notifier.recv().unwrap();
                for notifier in &notifiers {
                    notifier.notify(&execution_result);
                }
            }
        });

        event!(Level::INFO, "Initiating the main task loop");
        loop {
            thread::sleep(Duration::from_secs(1));
            {
                let mut task_queue = self.tasks.lock().unwrap();
                loop {
                    if task_queue
                        .peek()
                        .is_none_or(|t| t.0.next_execution_at() > &Zoned::now().datetime())
                    {
                        break;
                    }

                    let task = task_queue.dequeue();
                    let tx_task = tx_task.clone();
                    let tx_notifier = tx_notifier.clone();
                    tokio::spawn(async move {
                        executor::execute_check(task, tx_task, tx_notifier).await;
                    });
                }
            }
        }
    }
}
