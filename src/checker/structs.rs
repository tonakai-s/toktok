use std::{fmt::Display, str::FromStr};

use crate::parser::{ConfigKey, error::ConfigError, keys::ConfigKeyInvalidFormat};

#[derive(Debug)]
pub enum CheckerType {
    Web,
    Server,
}
impl Display for CheckerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckerType::Web => write!(f, "web"),
            CheckerType::Server => write!(f, "server"),
        }
    }
}
impl FromStr for CheckerType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "web" => Ok(CheckerType::Web),
            "server" => Ok(CheckerType::Server),
            _ => Err(format!("Is not a valid type: {s}")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CheckerStatus {
    Success,
    Error,
    Timeout,
}
#[derive(Debug)]
pub struct CheckerResult {
    pub service_name: String,
    pub status: CheckerStatus,
    pub message: String,
}
impl CheckerResult {
    pub fn new(service_name: String, status: CheckerStatus, message: String) -> Self {
        Self {
            service_name,
            status,
            message,
        }
    }
}
impl Display for CheckerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckerStatus::Success => write!(f, "Success"),
            CheckerStatus::Error => write!(f, "Error"),
            CheckerStatus::Timeout => write!(f, "Timeout"),
        }
    }
}
impl Display for CheckerResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Service: {} - Status: {} - Message: {}",
            self.service_name, self.status, self.message
        )
    }
}

#[derive(Debug)]
pub enum CheckerParseError {
    KeyNotFoundAt(ConfigKey, CheckerType),
    KeyNotFound(ConfigKey),
    InvalidType(String),
    InvalidFormat(ConfigKey, ConfigKeyInvalidFormat),
    InternalParse(String),
}
impl Display for CheckerParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckerParseError::KeyNotFoundAt(key, c_type) => {
                write!(f, "Key {key} is mandatory for a service of type {c_type}.")
            }
            CheckerParseError::KeyNotFound(key) => write!(f, "Key '{key}' is mandatory."),
            CheckerParseError::InvalidType(t) => write!(f, "Invalid type informed: {t}."),
            CheckerParseError::InvalidFormat(key, format) => {
                write!(f, "Invalid format for '{key}'. Expected: {format}")
            }
            CheckerParseError::InternalParse(e) => write!(f, "{e}"),
        }
    }
}

impl ConfigError for CheckerParseError {}
