use jiff::{SignedDuration, Zoned, civil::DateTime};

/// The informations about a task.
/// The `name` is defined by the key of the service in the yaml config file.
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
