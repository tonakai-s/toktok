use std::sync::{Arc, Mutex};

use crate::{
    executors::web_executor::WebExecutor, task::{ApplicationType, Task},
};

pub trait SpecificExecutor {
    fn new(task: Arc<Mutex<Task>>) -> Self;
    fn execute(&self) -> impl std::future::Future<Output = ()> + Send;
    fn task_reset(&self);
}

pub struct ExecutorFactory {}
impl ExecutorFactory {
    pub fn create(task: Arc<Mutex<Task>>) -> impl SpecificExecutor {
        match task.lock().unwrap().application_type() {
            ApplicationType::Web(_) => WebExecutor::new(task.clone()),
        }
    }
}
