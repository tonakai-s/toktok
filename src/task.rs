use jiff::{SignedDuration, Zoned, civil::DateTime};
use reqwest::StatusCode;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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
    secs_elapsed_since_last_execution: SignedDuration,
    last_execution_at: DateTime,
    application_type: ApplicationType,
    status: TaskStatus,
}

impl Task {
    const EXECUTION_ADDER: SignedDuration = SignedDuration::from_secs(1);
    const DEFAULT_ELAPSE_START: SignedDuration = SignedDuration::from_secs(0);

    pub fn new(
        interval: SignedDuration,
        secs_elapsed_since_last_execution: SignedDuration,
        last_execution_at: DateTime,
        application_type: ApplicationType,
        status: TaskStatus,
    ) -> Self {
        Self {
            interval,
            secs_elapsed_since_last_execution,
            last_execution_at,
            application_type,
            status,
        }
    }
    pub fn default_reset(&mut self) {
        self.last_execution_at = (Zoned::now()).datetime();
        self.secs_elapsed_since_last_execution = Task::DEFAULT_ELAPSE_START;
    }
    pub fn check_trigger(&mut self) -> bool {
        if self.status == TaskStatus::Running {
            return false;
        }
        if self.interval <= self.secs_elapsed_since_last_execution {
            true
        } else {
            self.secs_elapsed_since_last_execution += Task::EXECUTION_ADDER;
            false
        }
    }
    pub fn status(&self) -> &TaskStatus {
        &self.status
    }
    pub fn application_type(&self) -> &ApplicationType {
        &self.application_type
    }
    pub fn last_execution_at(&mut self, at: DateTime) {
        self.last_execution_at = at;
    }
}
