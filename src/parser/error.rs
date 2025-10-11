use std::{
    fmt::{Debug, Display},
    io,
};

pub trait ConfigError: Debug + Display {}
impl<E: ConfigError + 'static> From<E> for Box<dyn ConfigError> {
    fn from(e: E) -> Self {
        Box::new(e)
    }
}

#[derive(Debug)]
pub enum ConfigFileError {
    UnableToOpen(io::Error),
    UnableToRead(io::Error),
    UnableToScan(yaml_rust2::ScanError),
}
impl Display for ConfigFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigFileError::UnableToOpen(err) => {
                write!(f, "Unable to open the config file: {err}")
            }
            ConfigFileError::UnableToRead(err) => {
                write!(f, "Unable to read the config file: {err}")
            }
            ConfigFileError::UnableToScan(err) => {
                write!(f, "Unable to create a base scan of config file: {err}")
            }
        }
    }
}
impl ConfigError for ConfigFileError {}

#[derive(Debug)]
pub enum ConfigParseError {
    NoServiceProvided,
}
impl Display for ConfigParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigParseError::NoServiceProvided => {
                write!(f, "None service provided, aborting.")
            }
        }
    }
}
impl ConfigError for ConfigParseError {}
