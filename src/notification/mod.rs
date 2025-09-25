use crate::executor::ExecutionResult;

pub mod email;

pub trait Notifier {
    fn notify(&self, exec_result: &ExecutionResult);
}
