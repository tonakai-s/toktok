use std::io::Read;

use jiff::SignedDuration;
use yaml_rust2::{Yaml, YamlLoader};

use crate::{
    args::Args,
    checker::{error::CheckerParseError, Checker},
    notification::{email::MailNotifier, error::NotificationParseError},
    parser::{
        error::{ConfigFileError, ConfigParseError}, ConfigKey
    },
    task::Task,
    task_info::TaskInfo,
};

const DEFAULT_CONFIG_FILE: &str = "toktok.yaml";

/// The general struct for the config file after all validations.
/// Each new notification method is added as a field
#[derive(Default)]
pub struct Configuration {
    pub tasks: Vec<Task>,
    pub mailer: Option<MailNotifier>,
}

/// Responsible by the build proccess while reading the config file.
pub struct ConfigurationBuilder {
    config: Vec<Yaml>,
    tasks: Vec<Task>,
    mailer: Option<MailNotifier>,
}
impl ConfigurationBuilder {
    /// Create the base builder, this point also already read the config file,
    /// from the custom path by argument or the default filename in the same folder of the binary.
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

    /// Read all services inside the key `services` and parse each of them into a `Task`
    pub fn services(mut self) -> Result<Self, CheckerParseError> {
        let mut tasks = vec![];

        let services = &self.config[0]["services"];
        match services {
            Yaml::Hash(services) => {
                for service in services.iter() {
                    let service_name = service.0.as_str().unwrap().to_string();

                    let interval = self.interval(service.1)?;
                    let checker = self.get_checker(service.1)?;
                    let info = TaskInfo::new(service_name, interval);

                    tasks.push(Task::new(info, checker));
                }
            }
            _ => {
                return Err(CheckerParseError::InternalParse(
                    "Invalid services format or none was provided".to_string(),
                ));
            }
        };

        self.tasks = tasks;
        Ok(self)
    }

    fn get_checker(&self, service_attrs: &Yaml) -> Result<Checker, CheckerParseError> {
        let service_config = &service_attrs["configuration"];
        if service_config.is_badvalue() {
            return Err(CheckerParseError::KeyNotFound(ConfigKey::Configuration));
        }

        Checker::try_from(service_config)
    }

    fn interval(&self, service_attrs: &Yaml) -> Result<SignedDuration, CheckerParseError> {
        match &service_attrs[ConfigKey::Interval.as_ref()] {
            Yaml::Integer(inter) if *inter > 0 => Ok(SignedDuration::from_secs(*inter)),
            _ => Err(CheckerParseError::KeyNotFound(ConfigKey::Interval)),
        }
    }

    /// Parse the specific `notification->mailer` map into the config file.
    pub fn mailer(mut self) -> Result<Self, NotificationParseError> {
        let mailer_section = &self.config[0]["notification"]["mailer"];
        if !mailer_section.is_hash() {
            return Ok(self);
        }

        self.mailer = Some(MailNotifier::try_from(mailer_section)?);
        Ok(self)
    }

    /// Generate a `Configuration` struct after all validations passed.
    pub fn build(self) -> Result<Configuration, ConfigParseError> {
        if self.tasks.is_empty() {
            return Err(ConfigParseError::NoServiceProvided);
        }

        Ok(Configuration {
            tasks: self.tasks,
            mailer: self.mailer,
        })
    }
}

impl Configuration {
    pub fn builder(args: &Args) -> Result<ConfigurationBuilder, ConfigFileError> {
        ConfigurationBuilder::try_new(args)
    }

    pub fn has_tasks(&self) -> bool {
        !self.tasks.is_empty()
    }
}
