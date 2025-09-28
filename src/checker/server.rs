use std::{net::{SocketAddr, TcpStream}, str::FromStr, time::Duration};

use anyhow::{bail, Result};
use yaml_rust2::{yaml::Hash, Yaml};

use crate::checker::{structs::{CheckerResult, CheckerStatus}};

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

    pub fn parse_timeout(data: &Hash) -> Result<Option<Duration>> {
        let timeout = match data.get(&Yaml::String("timeout".into())) {
            Some(t)  => t.as_i64().unwrap(),
            None => return Ok(None)
        };

        if timeout <= 0 {
            bail!("Key 'timeout' must be a number greater than zero");
        }

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

impl TryFrom<&Hash> for ServerChecker {
    type Error = anyhow::Error;
    fn try_from(value: &Hash) -> Result<Self, Self::Error> {
        let socket = match value.get(&Yaml::String("socket".into())) {
            Some(socket) => socket.as_str().unwrap(),
            None => bail!("Key 'socket' is mandatory for service of type server"),
        };

        let socket_split = socket.split(':').collect::<Vec<&str>>();
        if socket_split.is_empty() {
            bail!("Key 'socket' must follow the pattern HOST:PORT");
        }

        let timeout = ServerChecker::parse_timeout(value)?;
        match SocketAddr::from_str(socket) {
            Ok(socket_addr) => Ok(ServerChecker::new(socket_addr, timeout)),
            Err(err) => bail!("Invalid socket at configuration: {}", err)
        }
    }
}