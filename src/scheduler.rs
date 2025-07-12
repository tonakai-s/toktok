use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::{data::Data, executor::execute, task::Task};

pub struct Scheduler {
    tasks: Vec<Arc<Mutex<(Task, Data)>>>,
}
impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }
    pub fn enqueue(&mut self, task: Task, data: Data) {
        self.tasks.push(Arc::new(Mutex::new((task, data))));
    }
    pub async fn start(&self) {
        let mut count = 0;
        loop {
            thread::sleep(Duration::from_secs(1));

            for task in &self.tasks {
                if !task.lock().unwrap().0.check_trigger() {
                    continue;
                }

                let c_task = task.clone();
                println!(
                    "--Lets spawn a task for '{}'!",
                    c_task.lock().unwrap().0.name()
                );
                tokio::spawn(async move {
                    let data = { c_task.lock().unwrap().1.clone() };
                    execute(&data).await;
                    c_task.lock().unwrap().0.default_reset();
                });
            }

            count += 1;
            println!("-Elapsed seconds: {count}");
        }
    }
}
