use std::{fmt::Display, sync::mpsc::Sender};

use tracing::{Level, event};

use crate::{
    checker::{Checker, WebChecker},
    task::Task,
};

#[derive(Debug, PartialEq, Eq)]
pub enum ExecutionStatus {
    Success,
    Error,
    Timeout,
}
#[derive(Debug)]
pub struct ExecutionResult {
    pub service_name: String,
    pub status: ExecutionStatus,
    pub message: String,
}
impl ExecutionResult {
    fn new(service_name: String, status: ExecutionStatus, message: String) -> Self {
        Self {
            service_name,
            status,
            message,
        }
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

pub async fn execute(mut task: Task, tx_task: Sender<Task>, tx_notifier: Sender<ExecutionResult>) {
    task.set_last_execution_at();
    let checker_result = match task.checker() {
        Checker::Web(web_data) => web_execute(&task.name(), &web_data).await,
    };
    task.log(&checker_result);
    if checker_result.status != ExecutionStatus::Success {
        if let Err(err) = tx_notifier.send(checker_result) {
            event!(
                Level::ERROR,
                message = ?err.0,
                error = %err,
                "Error sending the checker result to notifier thread"
            );
        }
    }
    if let Err(err) = tx_task.send(task) {
        event!(
            Level::ERROR,
            message = ?err.0,
            error = %err,
            "Error sending task to the enqueuer thread"
        );
    }
}

pub async fn web_execute(service: &str, data: &WebChecker) -> ExecutionResult {
    let response = reqwest::get(data.url()).await;
    match response {
        std::result::Result::Ok(response) => {
            if response.status() == *data.expected_code() {
                ExecutionResult::new(
                    service.to_string(),
                    ExecutionStatus::Success,
                    format!("Service available with status {}", response.status()),
                )
            } else {
                ExecutionResult::new(
                    service.to_string(),
                    ExecutionStatus::Error,
                    format!("Service unavailable with status {}", response.status()),
                )
            }
        }
        Err(err) => ExecutionResult::new(
            service.to_string(),
            ExecutionStatus::Error,
            format!("Service unavailable: {err}"),
        ),
    }
}
