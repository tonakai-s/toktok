use std::io::Read;

use jiff::SignedDuration;
use yaml_rust2::{Yaml, YamlLoader};

use crate::{
    args::Args,
    checker::{Checker, structs::CheckerParserError},
    notification::email::MailNotifier,
    parser::{ConfigKey, error::ConfigFileError},
    task::Task,
    task_info::TaskInfo,
};

const DEFAULT_CONFIG_FILE: &str = "toktok.yaml";

#[derive(Default)]
pub struct Configuration {
    pub tasks: Vec<Task>,
    pub mailer: Option<MailNotifier>,
}

pub struct ConfigurationBuilder {
    config: Vec<Yaml>,
    tasks: Vec<Task>,
    mailer: Option<MailNotifier>,
}
impl ConfigurationBuilder {
    fn try_new(args: &Args) -> Result<Self, ConfigFileError> {
        Ok(Self {
            config: ConfigurationBuilder::load_config(args.config.as_deref())?,
            tasks: vec![],
            mailer: None,
        })
    }

    fn load_config(file_path: Option<&str>) -> Result<Vec<Yaml>, ConfigFileError> {
        let mut content = String::new();
        let config_path = file_path.map_or(DEFAULT_CONFIG_FILE, |config| config);

        let mut file = std::fs::File::open(config_path).map_err(ConfigFileError::UnableToOpen)?;

        file.read_to_string(&mut content)
            .map_err(ConfigFileError::UnableToRead)?;

        YamlLoader::load_from_str(&content).map_err(ConfigFileError::UnableToScan)
    }

    pub fn services(mut self) -> Result<Self, String> {
        let mut tasks = vec![];

        let services = &self.config[0]["services"];
        match services {
            Yaml::Hash(services) => {
                for service in services.iter() {
                    let service_name = service.0.as_str().unwrap().to_string();

                    let interval = self.interval(service.1).map_err(|e| {
                        format!("{e}\nThrowed when reading service: {service_name}")
                    })?;
                    let checker = self.get_checker(service.1).map_err(|e| {
                        format!("{e}\nThrowed when reading service: {service_name}")
                    })?;
                    let info = TaskInfo::new(service_name, interval);

                    tasks.push(Task::new(info, checker));
                }
            }
            _ => return Err("Invalid services format or none was provided".to_string()),
        };

        self.tasks = tasks;
        Ok(self)
    }

    fn interval(&self, service_attrs: &Yaml) -> Result<SignedDuration, String> {
        match &service_attrs["interval"] {
            Yaml::Integer(inter) if *inter > 0 => Ok(SignedDuration::from_secs(*inter)),
            _ => Err(String::from(
                "'interval' is a mandatory map field for a service.",
            )),
        }
    }
    fn get_checker(&self, service_attrs: &Yaml) -> Result<Checker, CheckerParserError> {
        let service_config = &service_attrs["configuration"];
        if service_config.is_badvalue() {
            return Err(CheckerParserError::KeyNotFound(ConfigKey::Configuration));
        }

        Checker::try_from(service_config)
    }

    pub fn mailer(mut self) -> Result<Self, String> {
        let mailer_section = &self.config[0]["notification"]["mailer"];
        if !mailer_section.is_hash() {
            return Ok(self);
        }

        self.mailer = Some(MailNotifier::try_from(mailer_section)?);
        Ok(self)
    }

    pub fn build(self) -> Configuration {
        Configuration {
            tasks: self.tasks,
            mailer: self.mailer,
        }
    }
}

impl Configuration {
    pub fn builder(args: &Args) -> Result<ConfigurationBuilder, String> {
        ConfigurationBuilder::try_new(args).map_err(|e| format!("{e}"))
    }

    pub fn has_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }
}
