use std::sync::mpsc::Sender;

use tracing::{Level, event};

use crate::{
    checker::{structs::{CheckerResult, CheckerStatus}, Checker, WebChecker},
    task::Task,
};

pub async fn execute(mut task: Task, tx_task: Sender<Task>, tx_notifier: Sender<CheckerResult>) {
    task.set_last_execution_at();
    let checker_result = match task.checker() {
        Checker::Web(web_data) => web_execute(&task.name(), &web_data).await,
    };
    task.log(&checker_result);
    if checker_result.status != CheckerStatus::Success {
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

pub async fn web_execute(service: &str, data: &WebChecker) -> CheckerResult {
    let response = reqwest::get(data.url()).await;
    match response {
        std::result::Result::Ok(response) => {
            if response.status() == *data.expected_code() {
                CheckerResult::new(
                    service.to_string(),
                    CheckerStatus::Success,
                    format!("Service available with status {}", response.status()),
                )
            } else {
                CheckerResult::new(
                    service.to_string(),
                    CheckerStatus::Error,
                    format!("Service unavailable with status {}", response.status()),
                )
            }
        }
        Err(err) => CheckerResult::new(
            service.to_string(),
            CheckerStatus::Error,
            format!("Service unavailable: {err}"),
        ),
    }
}
