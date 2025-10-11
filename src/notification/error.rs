use std::{fmt::Display, io};

use crate::{
    notification::NotificationType,
    parser::{ConfigKey, error::ConfigError},
};

#[derive(Debug)]
pub enum NotificationParseError {
    KeyNotFoundAt(ConfigKey, NotificationType),
    InvalidFormat(ConfigKey, String),
    InternalParse(String),
    InternalBuild(String),
    UnableToReadFile(String, io::Error),
    UnableToOpenFile(String, io::Error),
}
impl Display for NotificationParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationParseError::KeyNotFoundAt(key, c_type) => {
                write!(
                    f,
                    "Key '{key}' is mandatory for a notification of type {c_type}."
                )
            }
            NotificationParseError::InvalidFormat(key, format) => {
                write!(f, "Invalid format for '{key}'. Expected: {format}")
            }
            NotificationParseError::InternalParse(e) => write!(f, "{e}"),
            NotificationParseError::InternalBuild(e) => write!(f, "{e}"),
            NotificationParseError::UnableToReadFile(path, e) => write!(
                f,
                "Unable to read the desired file.\nPath: {path}\nError: {e}"
            ),
            NotificationParseError::UnableToOpenFile(path, e) => write!(
                f,
                "Unable to open the desired file.\nPath: {path}\nError: {e}"
            ),
        }
    }
}
impl ConfigError for NotificationParseError {}
