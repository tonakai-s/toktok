use std::{thread, time::Duration};

use jiff::{
    SignedDuration,
    civil::{Date, DateTime},
};
use reqwest::StatusCode;

use crate::executors::factory::{ExecutorFactory, SpecificExecutor};

#[derive(Debug, Copy, Clone)]
pub enum TaskStatus {
    Waiting,
    Running,
}

#[derive(Debug, Clone)]
pub enum ApplicationType {
    Web(WebApplicationType),
}

#[derive(Debug, Clone)]
pub struct WebApplicationType {
    domain: String,
    path: Option<String>,
    expected_code: StatusCode,
}
impl WebApplicationType {
    pub fn new(domain: String, path: Option<String>, expected_code: StatusCode) -> Self {
        Self {
            domain,
            path,
            expected_code,
        }
    }
    pub fn url(&self) -> String {
        if let Some(path) = &self.path {
            // TODO: Check slash presence
            format!("{}{}", self.domain, path)
        } else {
            self.domain.clone()
        }
    }
    pub fn expected_code(&self) -> &StatusCode {
        &self.expected_code
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    interval: SignedDuration,
    last_execution_at: DateTime,
    application_type: ApplicationType,
    status: TaskStatus,
}

impl Task {
    pub fn new(
        interval: SignedDuration,
        last_execution_at: DateTime,
        application_type: ApplicationType,
        status: TaskStatus,
    ) -> Self {
        Self {
            interval,
            last_execution_at,
            application_type,
            status,
        }
    }
    pub fn status(&self) -> &TaskStatus {
        &self.status
    }
    pub fn application_type(&self) -> &ApplicationType {
        &self.application_type
    }
}

pub struct Scheduler {
    tasks: Vec<Task>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }
    pub fn enqueue(&mut self, task: Task) {
        self.tasks.push(task);
    }
    pub async fn start(&self) {
        loop {
            thread::sleep(Duration::from_secs(1));
            for task in &self.tasks {
                let c_task = task.clone();
                tokio::spawn(async move {
                    let executor = ExecutorFactory::create(c_task);
                    executor.execute().await;
                });
            }
        }
    }
}
