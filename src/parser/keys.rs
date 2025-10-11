use core::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum ConfigKey {
    // General service
    Service,
    Interval,
    Timeout,
    Configuration,
    Type,
    // Service type web
    Url,
    ExpectedHttpCode,
    Headers,
    // Service type server
    Socket,
    // General notification
    Notification,
    // Notification email
    SmtpDomain,
    SmtpCredentials,
    MailFrom,
    MailTo,
    MailCc,
    MailBcc,
}
impl AsRef<str> for ConfigKey {
    fn as_ref(&self) -> &str {
        match self {
            ConfigKey::Service => "service",
            ConfigKey::Interval => "interval",
            ConfigKey::Url => "url",
            ConfigKey::ExpectedHttpCode => "expected_http_code",
            ConfigKey::Headers => "headers",
            ConfigKey::Timeout => "timeout",
            ConfigKey::Configuration => "configuration",
            ConfigKey::Type => "type",
            ConfigKey::Socket => "socket",
            ConfigKey::Notification => "notification",
            ConfigKey::SmtpDomain => "smtp_domain",
            ConfigKey::SmtpCredentials => "smtp_credentials",
            ConfigKey::MailFrom => "from",
            ConfigKey::MailTo => "to",
            ConfigKey::MailCc => "cc",
            ConfigKey::MailBcc => "bcc",
        }
    }
}
impl Display for ConfigKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigKey::Service => write!(f, "service"),
            ConfigKey::Interval => write!(f, "interval"),
            ConfigKey::Url => write!(f, "url"),
            ConfigKey::ExpectedHttpCode => write!(f, "expected_http_code"),
            ConfigKey::Headers => write!(f, "headers"),
            ConfigKey::Timeout => write!(f, "timeout"),
            ConfigKey::Configuration => write!(f, "configuration"),
            ConfigKey::Type => write!(f, "type"),
            ConfigKey::Socket => write!(f, "socket"),
            ConfigKey::Notification => write!(f, "notification"),
            ConfigKey::SmtpDomain => write!(f, "smtp_domain"),
            ConfigKey::SmtpCredentials => write!(f, "smtp_credentials"),
            ConfigKey::MailFrom => write!(f, "from"),
            ConfigKey::MailTo => write!(f, "to"),
            ConfigKey::MailCc => write!(f, "cc"),
            ConfigKey::MailBcc => write!(f, "bcc"),
        }
    }
}

// TODO: Improve this invalid format Display implementation
#[derive(Debug)]
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
