use std::{fmt::Display, sync::mpsc::Sender};

use crate::{
    checker::{Checker, WebChecker},
    task::Task,
};

pub enum ExecutionStatus {
    Success,
    Error,
    Timeout,
}
pub struct ExecutionResult {
    pub status: ExecutionStatus,
    pub message: String,
}
impl ExecutionResult {
    fn new(status: ExecutionStatus, message: String) -> Self {
        Self { status, message }
    }
}
impl Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStatus::Success => write!(f, "Success"),
            ExecutionStatus::Error => write!(f, "Error"),
            ExecutionStatus::Timeout => write!(f, "Timeout"),
        }
    }
}

pub async fn execute(mut task: Task, tx: Sender<Task>) {
    task.set_last_execution_at();
    let checker_result = match task.checker() {
        Checker::Web(web_data) => web_execute(&web_data).await,
    };
    task.log(checker_result);
    let _ = tx.send(task);
}

pub async fn web_execute(data: &WebChecker) -> ExecutionResult {
    let response = reqwest::get(data.url()).await;
    match response {
        std::result::Result::Ok(response) => {
            if response.status() == *data.expected_code() {
                ExecutionResult::new(
                    ExecutionStatus::Success,
                    format!("Service available with status {}", response.status()),
                )
            } else {
                ExecutionResult::new(
                    ExecutionStatus::Error,
                    format!("Service unavailable with status {}", response.status()),
                )
            }
        }
        Err(err) => ExecutionResult::new(
            ExecutionStatus::Error,
            format!("Service unavailable with error {err}"),
        ),
    }
}
