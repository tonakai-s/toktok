use std::{str::FromStr, time::Duration};

use yaml_rust2::Yaml;

pub mod server;
pub mod structs;
pub mod web;
pub mod error;

pub use server::ServerChecker;
pub use web::WebChecker;

use crate::{
    checker::{error::CheckerParseError, structs::CheckerType},
    parser::{keys::ConfigKeyInvalidFormat, ConfigKey},
};

#[derive(Debug)]
pub enum Checker {
    Web(Box<WebChecker>),
    Server(ServerChecker),
}

impl Checker {
    pub fn timeout(service_attrs: &Yaml) -> Result<Option<Duration>, CheckerParseError> {
        match &service_attrs[ConfigKey::Timeout.as_ref()] {
            Yaml::Integer(time) if *time > 0 => {
                let time_u64 = TryInto::<u64>::try_into(*time).map_err(|_| {
                    CheckerParseError::InvalidFormat(
                        ConfigKey::Timeout,
                        ConfigKeyInvalidFormat::new(ConfigKey::Timeout),
                    )
                })?;
                Ok(Some(Duration::from_secs(time_u64)))
            }
            _ => Ok(None),
        }
    }
}

impl TryFrom<&Yaml> for Checker {
    type Error = CheckerParseError;

    fn try_from(config: &Yaml) -> Result<Self, Self::Error> {
        let service_type = match &config[ConfigKey::Type.as_ref()] {
            Yaml::String(serv_type) => {
                CheckerType::from_str(serv_type).map_err(CheckerParseError::InternalParse)?
            }
            _ => return Err(CheckerParseError::KeyNotFound(ConfigKey::Type)),
        };

        match service_type {
            CheckerType::Web => {
                let web_checker = WebChecker::try_from(config)?;
                Ok(Checker::Web(Box::new(web_checker)))
            }
            CheckerType::Server => {
                let server_checker = ServerChecker::try_from(config)?;
                Ok(Checker::Server(server_checker))
            }
        }
    }
}
