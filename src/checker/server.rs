use std::{
    net::{SocketAddr, TcpStream},
    str::FromStr,
    time::Duration,
};

use yaml_rust2::Yaml;

use crate::checker::{
    Checker,
    structs::{CheckerResult, CheckerStatus},
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
    type Error = String;
    fn try_from(data: &Yaml) -> Result<Self, Self::Error> {
        let socket = match &data["socket"] {
            Yaml::String(s) if !s.is_empty() => s,
            _ => {
                return Err(String::from(
                    "Key 'socket' is mandatory for service of type server",
                ));
            }
        };

        let socket_split = socket.split(':').collect::<Vec<&str>>();
        if socket_split.is_empty() {
            return Err(String::from(
                "Key 'socket' must follow the pattern HOST:PORT",
            ));
        }

        let timeout = Checker::timeout(data)?;
        let socket_addr =
            SocketAddr::from_str(socket).map_err(|e| format!("Invalid socket: {e}"))?;

        Ok(ServerChecker::new(socket_addr, timeout))
    }
}
