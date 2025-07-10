use reqwest::StatusCode;

use crate::{
    executors::web_executor::WebExecutor,
    scheduler::{ApplicationType, Task},
};

pub trait SpecificExecutor {
    fn new(task: Task, url: String, expected_code: StatusCode) -> Self;
    fn execute(&self) -> impl std::future::Future<Output = ()> + Send;
}

pub struct ExecutorFactory {}
impl ExecutorFactory {
    pub fn create(task: Task) -> impl SpecificExecutor {
        match task.application_type() {
            ApplicationType::Web(web_data) => {
                WebExecutor::new(task.clone(), web_data.url(), *web_data.expected_code())
            }
        }
    }
}
