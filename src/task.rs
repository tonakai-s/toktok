use jiff::{Zoned, civil::DateTime};

use crate::{
    checker::{structs::CheckerResult, Checker}, task_info::TaskInfo, task_logger::TaskLogger,
};

#[derive(Debug)]
pub struct Task {
    info: TaskInfo,
    checker: Checker,
    logger: TaskLogger,
}

impl Task {
    pub fn new(info: TaskInfo, checker: Checker) -> Self {
        let logger = match TaskLogger::new(info.name.clone(), None) {
            Ok(tl) => tl,
            Err(err) => panic!("{}", err),
        };
        Self {
            info,
            checker,
            logger,
        }
    }

    pub fn name(&self) -> String {
        self.info.name.clone().to_string()
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

    pub fn log(&mut self, exec_result: &CheckerResult) {
        self.logger.log(&exec_result);
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
