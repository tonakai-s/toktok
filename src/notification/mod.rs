use std::fmt::Display;

use crate::checker::structs::CheckerResult;

pub mod email;
pub mod error;

pub trait Notifier {
    fn notify(&self, exec_result: &CheckerResult);
}

#[derive(Debug)]
pub enum NotificationType {
    Email,
}
impl Display for NotificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationType::Email => write!(f, "email"),
        }
    }
}
