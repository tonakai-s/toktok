use anyhow::bail;
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
                Ok(Checker::Web(Box::new(web_checker)))
            }
            Some("server") => {
                let server_checker = ServerChecker::try_from(config)?;
                Ok(Checker::Server(server_checker))
            }
            _ => bail!(
                "Type '{}' is not valid",
                service_type.as_str().unwrap_or("undefined")
            ),
        }
    }
}
