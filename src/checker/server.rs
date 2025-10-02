use std::{net::{SocketAddr, TcpStream}, str::FromStr, time::Duration};

use anyhow::{bail, Result};
use yaml_rust2::Yaml;

use crate::{checker::structs::{CheckerResult, CheckerStatus}, configuration::ConfigurationParseError};

#[derive(Debug, PartialEq, Eq)]
pub struct ServerChecker {
    host: SocketAddr,
    timeout: Option<Duration>
}

impl ServerChecker {
    pub fn new(
        host: SocketAddr,
        timeout: Option<Duration>
    ) -> Self {
        Self { host, timeout }
    }

    pub fn parse_timeout(data: &Yaml) -> Result<Option<Duration>> {
        let timeout = match data["timeout"] {
            Yaml::BadValue => return Ok(None),
            Yaml::Integer(t) if t > 0 => t,
            _ => return Ok(None)
        };

        Ok(Some(Duration::from_secs_f64(timeout as f64)))
    }

    pub async fn check(&self, service: &str) -> CheckerResult {
        let stream = if self.timeout.is_some() {
            TcpStream::connect_timeout(&self.host, self.timeout.unwrap())
        } else {
            TcpStream::connect(self.host.clone())
        };
        match stream {
            Ok(_) => {
                CheckerResult::new(
                    service.to_string(),
                    CheckerStatus::Success,
                    "Server connected successfully via TCP/IP".into(),
                )
            }
            Err(err) => CheckerResult::new(
                service.to_string(),
                CheckerStatus::Error,
                format!("Server unavailable: {err}"),
            ),
        }
    }
}

impl TryFrom<&Yaml> for ServerChecker {
    type Error = anyhow::Error;
    fn try_from(data: &Yaml) -> Result<Self, Self::Error> {
        let socket = match &data["socket"] {
            Yaml::String(s) if !s.is_empty() => s,
            _ => bail!(ConfigurationParseError::KeyNotFound("socket", "at service of type server and cannot be empty")),
        };

        let socket_split = socket.split(':').collect::<Vec<&str>>();
        if socket_split.is_empty() {
            bail!("Key 'socket' must follow the pattern HOST:PORT");
        }

        let timeout = ServerChecker::parse_timeout(data)?;
        match SocketAddr::from_str(&socket) {
            Ok(socket_addr) => Ok(ServerChecker::new(socket_addr, timeout)),
            Err(err) => bail!("Invalid socket at configuration: {}", err)
        }
    }
}