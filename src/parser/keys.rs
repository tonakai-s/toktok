use core::fmt;
use std::fmt::Display;

pub enum ConfigKey {
    Url,
    ExpectedHttpCode,
    Headers,
    Timeout,
    Configuration,
    Type,
    Socket,
}
impl AsRef<str> for ConfigKey {
    fn as_ref(&self) -> &str {
        match self {
            ConfigKey::Url => "url",
            ConfigKey::ExpectedHttpCode => "expected_http_code",
            ConfigKey::Headers => "headers",
            ConfigKey::Timeout => "timeout",
            ConfigKey::Configuration => "configuration",
            ConfigKey::Type => "type",
            ConfigKey::Socket => "socket",
        }
    }
}
impl Display for ConfigKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigKey::Url => write!(f, "url"),
            ConfigKey::ExpectedHttpCode => write!(f, "expected_http_code"),
            ConfigKey::Headers => write!(f, "headers"),
            ConfigKey::Timeout => write!(f, "timeout"),
            ConfigKey::Configuration => write!(f, "configuration"),
            ConfigKey::Type => write!(f, "type"),
            ConfigKey::Socket => write!(f, "socket"),
        }
    }
}

// TODO: Improve this invalid format Display implementation
pub struct ConfigKeyInvalidFormat(ConfigKey);
impl ConfigKeyInvalidFormat {
    pub fn new(key: ConfigKey) -> Self {
        Self(key)
    }
}

impl Display for ConfigKeyInvalidFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            ConfigKey::ExpectedHttpCode => write!(f, "Must be a valid HTTP Code"),
            ConfigKey::Timeout => write!(f, "Must be a number greater than zero"),
            ConfigKey::Socket => write!(
                f,
                "Must be a value with pattern IP_ADDRESS:PORT or DOMAIN:PORT"
            ),
            _ => write!(f, "Undefined format"),
        }
    }
}
