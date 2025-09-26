use anyhow::bail;
use yaml_rust2::{yaml::Hash, Yaml};

pub mod structs;
pub mod web;
pub use web::WebChecker;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Checker {
    Web(WebChecker),
}

impl TryFrom<&Hash> for Checker {
    type Error = anyhow::Error;

    fn try_from(data_config: &Hash) -> Result<Self, Self::Error> {
        let service_type = match data_config.get(&Yaml::String("type".into())) {
            Some(t) => t.as_str().unwrap().to_lowercase(),
            None => bail!("'type' is mandatory field for a service"),
        };

        match service_type.as_str() {
            "web" => {
                let web_checker = WebChecker::try_from(data_config)?;
                Ok(Checker::Web(web_checker))
            }
            _ => bail!("Type '{}' is not valid", service_type),
        }
    }
}