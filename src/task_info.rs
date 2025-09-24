use jiff::{SignedDuration, Zoned, civil::DateTime};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskInfo {
    pub name: String,
    pub interval: SignedDuration,
    pub last_execution_at: DateTime,
    pub next_execution_at: DateTime,
}

impl TaskInfo {
    pub fn new(name: String, interval: SignedDuration) -> Self {
        Self {
            name,
            interval,
            last_execution_at: Zoned::now().datetime(),
            next_execution_at: Zoned::now().datetime(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TaskState {
    Waiting,
    Running,
}
