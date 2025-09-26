use crate::checker::structs::CheckerResult;

pub mod email;

pub trait Notifier {
    fn notify(&self, exec_result: &CheckerResult);
}
