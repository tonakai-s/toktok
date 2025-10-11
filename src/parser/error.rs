use std::{fmt::Display, io};

trait Error {}
pub struct ConfigError<T: Error + Display>(T);

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
