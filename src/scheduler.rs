use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{
    executors::factory::{ExecutorFactory, SpecificExecutor},
    task::Task,
};

pub struct Scheduler {
    tasks: Vec<Arc<Mutex<Task>>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Scheduler::new()
    }
}
impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }
    pub fn enqueue(&mut self, task: Task) {
        self.tasks.push(Arc::new(Mutex::new(task)));
    }
    pub async fn start(&self) {
        let mut count = 0;
        loop {
            for task in &self.tasks {
                if !task.lock().unwrap().check_trigger() {
                    continue;
                }

                let c_task = task.clone();
                println!("--Lets spawn a task!");
                tokio::spawn(async move {
                    let executor = ExecutorFactory::create(c_task);
                    executor.execute().await;
                    executor.task_reset();
                });
            }
            thread::sleep(Duration::from_secs(1));
            count += 1;
            println!("-Elapsed seconds: {count}");
        }
    }
}
