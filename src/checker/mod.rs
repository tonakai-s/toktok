use std::time::Duration;

use yaml_rust2::Yaml;

pub mod server;
pub mod structs;
pub mod web;

pub use server::ServerChecker;
pub use web::WebChecker;

#[derive(Debug)]
pub enum Checker {
    Web(Box<WebChecker>),
    Server(ServerChecker),
}

impl Checker {
    pub fn timeout(service_attrs: &Yaml) -> Result<Option<Duration>, String> {
        match &service_attrs["timeout"] {
            Yaml::Integer(time) if *time > 0 => {
                let time_u64 = TryInto::<u64>::try_into(*time)
                    .map_err(|e| format!("The defined timeout is not valid: {e}"))?;
                Ok(Some(Duration::from_secs(time_u64)))
            }
            _ => Ok(None),
        }
    }
}

impl TryFrom<&Yaml> for Checker {
    type Error = String;

    fn try_from(config: &Yaml) -> Result<Self, Self::Error> {
        let service_type = match &config["type"] {
            Yaml::String(serv_type) => serv_type,
            _ => return Err(String::from("'type' field is mandatory")),
        };

        match service_type.as_str() {
            "web" => {
                let web_checker = WebChecker::try_from(config)?;
                Ok(Checker::Web(Box::new(web_checker)))
            }
            "server" => {
                let server_checker = ServerChecker::try_from(config)?;
                Ok(Checker::Server(server_checker))
            }
            _ => Err(format!("The type {service_type} is not valid")),
        }
    }
}
