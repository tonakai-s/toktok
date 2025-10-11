use std::fmt::Display;

use crate::{checker::structs::CheckerType, parser::{error::ConfigError, keys::ConfigKeyInvalidFormat, ConfigKey}};

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