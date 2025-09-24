use std::sync::mpsc::Sender;

use crate::{
    checker::{Checker, WebChecker},
    task::Task,
};

pub async fn execute(mut task: Task, tx: Sender<Task>) {
    task.set_last_execution_at();
    match task.checker() {
        Checker::Web(web_data) => web_execute(&web_data).await,
    }
    let _ = tx.send(task);
}

pub async fn web_execute(data: &WebChecker) {
    let response = reqwest::get(data.url()).await;
    match response {
        std::result::Result::Ok(response) => {
            if response.status() == *data.expected_code() {
                println!("Service available with status {}", response.status());
                // Ok(response.status())
            } else {
                println!("Service unavailable with status {}", response.status());
                // bail!(response.status())
            }
        }
        Err(err) => {
            println!("Service unavailable with error {err}");
            // bail!(err)
        }
    }
}
