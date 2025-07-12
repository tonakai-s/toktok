use jiff::{SignedDuration, Zoned, civil::DateTime};

const EXECUTION_ADDER: SignedDuration = SignedDuration::from_secs(1);
const DEFAULT_ELAPSE_START: SignedDuration = SignedDuration::from_secs(0);

#[derive(Debug, Clone)]
pub struct Task {
    name: String,
    interval: SignedDuration,
    secs_elapsed_since_last_execution: SignedDuration,
    last_execution_at: DateTime,
    state: TaskState,
}

impl Task {
    pub fn new(name: String, interval: SignedDuration) -> Self {
        Self {
            name,
            interval,
            secs_elapsed_since_last_execution: interval,
            last_execution_at: Zoned::now().datetime(),
            state: TaskState::Waiting,
        }
    }
    pub fn check_trigger(&mut self) -> bool {
        if self.state == TaskState::Running {
            return false;
        }
        if self.interval <= self.secs_elapsed_since_last_execution {
            self.state = TaskState::Running;
            true
        } else {
            self.secs_elapsed_since_last_execution += EXECUTION_ADDER;
            false
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn default_reset(&mut self) {
        self.last_execution_at = (Zoned::now()).datetime();
        self.secs_elapsed_since_last_execution = DEFAULT_ELAPSE_START;
        self.state = TaskState::Waiting;
    }
    pub fn state(&self) -> &TaskState {
        &self.state
    }
    pub fn last_execution_at(&mut self, at: DateTime) {
        self.last_execution_at = at;
    }
    pub fn toggle_state(&mut self) {
        match self.state {
            TaskState::Running => self.state = TaskState::Waiting,
            TaskState::Waiting => self.state = TaskState::Running,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TaskState {
    Waiting,
    Running,
}

// #[derive(Debug, Clone)]
// pub struct Task<E: SpecificExecutor + Sized + Send + Sync> {
//     name: String,
//     interval: SignedDuration,
//     secs_elapsed_since_last_execution: SignedDuration,
//     last_execution_at: DateTime,
//     state: TaskState,
//     pub executor: E,
// }
// impl<E: 'static + Send + Sync + std::fmt::Debug + SpecificExecutor> TTask for Task<E> {
// }

// impl Task<WebExecutor> {
//     pub fn new(
//         name: String,
//         interval: SignedDuration,
//         last_execution_at: DateTime,
//         domain: String,
//         path: Option<String>,
//         expected_code: StatusCode,
//     ) -> Self {
//         let executor = WebExecutor::new(
//             domain, path, expected_code
//         );
//         Self {
//             name,
//             interval,
//             secs_elapsed_since_last_execution: interval,
//             last_execution_at,
//             state: TaskState::Waiting,
//             executor,
//         }
//     }
// }

// impl<E: SpecificExecutor + Sized + Send + Sync> Task<E> {
//     pub fn name(&self) -> &str {
//         &self.name
//     }
//     pub fn default_reset(&mut self) {
//         self.last_execution_at = (Zoned::now()).datetime();
//         self.secs_elapsed_since_last_execution = DEFAULT_ELAPSE_START;
//         self.state = TaskState::Waiting;
//     }
//     pub fn state(&self) -> &TaskState {
//         &self.state
//     }
//     pub fn last_execution_at(&mut self, at: DateTime) {
//         self.last_execution_at = at;
//     }
//     pub fn toggle_state(&mut self) {
//         match self.state {
//             TaskState::Running => self.state = TaskState::Waiting,
//             TaskState::Waiting => self.state = TaskState::Running,
//         }
//     }
// }
