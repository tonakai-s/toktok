use std::{
    net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs},
    str::FromStr,
    time::Duration,
};

use yaml_rust2::Yaml;

use crate::{
    checker::{
        Checker,
        structs::{CheckerParseError, CheckerResult, CheckerStatus, CheckerType},
    },
    parser::{ConfigKey, keys::ConfigKeyInvalidFormat},
};

#[derive(Debug, PartialEq, Eq)]
pub struct ServerChecker {
    host: SocketAddr,
    timeout: Option<Duration>,
}

impl ServerChecker {
    pub fn new(host: SocketAddr, timeout: Option<Duration>) -> Self {
        Self { host, timeout }
    }

    pub async fn check(&self, service: &str) -> CheckerResult {
        let stream = if self.timeout.is_some() {
            TcpStream::connect_timeout(&self.host, self.timeout.unwrap())
        } else {
            TcpStream::connect(self.host)
        };
        match stream {
            Ok(_) => CheckerResult::new(
                service.to_string(),
                CheckerStatus::Success,
                "Server connected successfully via TCP/IP".into(),
            ),
            Err(err) => CheckerResult::new(
                service.to_string(),
                CheckerStatus::Error,
                format!("Server unavailable: {err}"),
            ),
        }
    }
}

impl TryFrom<&Yaml> for ServerChecker {
    type Error = CheckerParseError;
    fn try_from(data: &Yaml) -> Result<Self, Self::Error> {
        let socket = match &data["socket"] {
            Yaml::String(s) if !s.is_empty() => s,
            _ => {
                return Err(CheckerParseError::KeyNotFoundAt(
                    ConfigKey::Socket,
                    CheckerType::Server,
                ));
            }
        };

        let socket_split = socket.split_once(':');
        if socket_split.is_none() {
            return Err(CheckerParseError::InvalidFormat(
                ConfigKey::Socket,
                ConfigKeyInvalidFormat::new(ConfigKey::Socket),
            ));
        }

        let addr = socket_split.unwrap().0;
        let socket_addr = if addr.parse::<IpAddr>().is_ok() {
            SocketAddr::from_str(socket)
                .map_err(|e| CheckerParseError::InternalParse(format!("Invalid socket: {e}")))?
        } else {
            let ip_addrs = socket.to_socket_addrs().map_err(|e| {
                CheckerParseError::InternalParse(format!(
                    "Unable to convert the socket to a Address due to the following error: {e}"
                ))
            })?;
            match ip_addrs.collect::<Vec<SocketAddr>>().first() {
                Some(s_addr) => *s_addr,
                None => {
                    return Err(CheckerParseError::InternalParse(
                        "No IP resolution found to the socket".to_string(),
                    ));
                }
            }
        };
        let timeout = Checker::timeout(data)?;

        Ok(ServerChecker::new(socket_addr, timeout))
    }
}
