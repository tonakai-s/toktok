use std::sync::{Arc, Mutex};

use crate::{
    executors::factory::SpecificExecutor,
    task::{ApplicationType, Task},
};

pub struct WebExecutor {
    task: Arc<Mutex<Task>>,
}

impl SpecificExecutor for WebExecutor {
    fn new(task: Arc<Mutex<Task>>) -> Self {
        Self { task }
    }
    async fn execute(&self) {
        let web_data = {
            let lock = self.task.lock().unwrap();
            match lock.application_type() {
                ApplicationType::Web(web_data) => web_data.clone(),
            }
        };

        println!("---Starting a Web Executor request NOW!");
        let response = reqwest::get(&web_data.url()).await;
        match response {
            Ok(response) => {
                if response.status() != *web_data.expected_code() {
                    println!("----Service unavailable with status {}!", response.status());
                } else {
                    println!("----Service available with status {}!", response.status());
                }
            }
            Err(err) => {
                println!("----Service unavailable. Error: {:#?}", err);
            }
        }
    }
    fn task_reset(&self) {
        self.task.lock().unwrap().default_reset();
    }
}
