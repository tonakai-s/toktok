use anyhow::bail;
use yaml_rust2::Yaml;

pub mod structs;
pub mod web;
pub mod server;

pub use web::WebChecker;
pub use server::ServerChecker;

#[derive(Debug)]
pub enum Checker {
    Web(WebChecker),
    Server(ServerChecker),
}

impl TryFrom<&Yaml> for Checker {
    type Error = anyhow::Error;

    fn try_from(config: &Yaml) -> Result<Self, Self::Error> {
        let service_type = &config["type"];
        if service_type.is_badvalue() {
            bail!("'type' is mandatory field for a service");
        }

        match service_type.as_str() {
            Some("web") => {
                let web_checker = WebChecker::try_from(config)?;
                Ok(Checker::Web(web_checker))
            },
            Some("server") => {
                let server_checker = ServerChecker::try_from(config)?;
                Ok(Checker::Server(server_checker))
            }
            _ => bail!("Type '{}' is not valid", service_type.as_str().unwrap_or("undefined")),
        }
    }
}