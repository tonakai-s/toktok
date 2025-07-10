use reqwest::StatusCode;

use crate::{executors::factory::SpecificExecutor, scheduler::Task};

pub struct WebExecutor {
    task: Task,
    url: String,
    expected_code: StatusCode,
}

impl SpecificExecutor for WebExecutor {
    fn new(task: Task, url: String, expected_code: StatusCode) -> Self {
        Self {
            task,
            url,
            expected_code,
        }
    }
    async fn execute(&self) {
        let response = reqwest::get(&self.url).await;
        match response {
            Ok(response) => {
                if response.status() != self.expected_code {
                    println!("Service unavailable with status {}!", response.status());
                } else {
                    println!("Service available with status {}!", response.status());
                }
            },
            Err(err) => {
                println!("Service unavailable. Error: {:#?}", err);
            }
        }
    }
}
