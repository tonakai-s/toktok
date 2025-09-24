use jiff::{Zoned, civil::DateTime};

use crate::{checker::Checker, task_info::TaskInfo};

#[derive(Debug)]
pub struct Task {
    info: TaskInfo,
    checker: Checker,
}

impl Task {
    pub fn name(&self) -> String {
        self.info.name.clone().to_string()
    }

    pub fn new(info: TaskInfo, checker: Checker) -> Self {
        Self { info, checker }
    }

    pub fn set_last_execution_at(&mut self) {
        self.info.last_execution_at = Zoned::now().datetime();
    }
    pub fn set_next_execution_at(&mut self) {
        self.info.next_execution_at = Zoned::now().datetime() + self.info.interval;
    }
    pub fn next_execution_at(&self) -> &DateTime {
        &self.info.next_execution_at
    }
    pub fn checker(&self) -> &Checker {
        &self.checker
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.next_execution_at().cmp(&other.next_execution_at())
    }
}
impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.next_execution_at() == other.next_execution_at()
    }
}

impl Eq for Task {}
