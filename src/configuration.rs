use std::{fmt::Display, io::{self, Read}};

use anyhow::bail;
use jiff::SignedDuration;
use yaml_rust2::{ScanError, Yaml, YamlLoader, yaml::Hash};

use crate::{checker::Checker, notification::email::MailNotifier, task::Task, task_info::TaskInfo};

#[derive(Debug)]
pub enum ConfigurationFileError {
    UnableToOpen(io::Error),
    UnableToRead(io::Error),
    UnableToScan(ScanError),
}
impl Display for ConfigurationFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigurationFileError::UnableToOpen(err) => write!(f, "Error trying to open the config file: {:?}", err),
            ConfigurationFileError::UnableToRead(err) => write!(f, "Error trying to read the config file: {:?}", err),
            ConfigurationFileError::UnableToScan(err) => write!(f, "Error interpreting the basic structure of config file: {:?}", err),
        }
    }
}

pub struct Configuration {
    pub tasks: Vec<Task>,
    pub mailer: Option<MailNotifier>,
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            tasks: vec![],
            mailer: None,
        }
    }
}

pub fn load_config() -> anyhow::Result<Configuration> {
    let mut content = String::new();
    let mut file = match std::fs::File::open("toktok.yaml") {
        Ok(f) => f,
        Err(err) => bail!(ConfigurationFileError::UnableToOpen(err)),
    };
    match file.read_to_string(&mut content) {
        Ok(_) => {}
        Err(err) => bail!(ConfigurationFileError::UnableToRead(err)),
    };
    let mut config = match YamlLoader::load_from_str(&content) {
        Ok(c) => c,
        Err(err) => bail!(ConfigurationFileError::UnableToScan(err)),
    };

    parse_config(&mut config)
}

fn parse_config(config: &mut [Yaml]) -> anyhow::Result<Configuration> {
    let mut configuration = Configuration::default();

    let config_hash = config[0].as_hash().unwrap();
    if let Some(notification_section) = config_hash.get(&Yaml::String("notification".into())) {
        parse_notifications(notification_section, &mut configuration)?;
    }

    for section in config[0].as_hash().unwrap().iter() {
        if section.0.as_str().unwrap() == "services" {
            configuration.tasks = parse_services(&section)?;
        }
    }

    Ok(configuration)
}

fn parse_notifications(section: &Yaml, configuration: &mut Configuration) -> anyhow::Result<()> {
    let section_hash = match section.as_hash() {
        Some(hash) => hash,
        None => bail!("The 'notification' is not a valid map"),
    };

    if let Some(mailer) = section_hash.get(&Yaml::String("mailer".into())) {
        let mailer_hash = match mailer.as_hash() {
            Some(hash) => hash,
            None => bail!("The 'mailer' is not a valid map"),
        };
        let mailer = MailNotifier::try_from(mailer_hash)?;

        configuration.mailer = Some(mailer);
    }

    Ok(())
}

fn parse_services(section: &(&Yaml, &Yaml)) -> anyhow::Result<Vec<Task>> {
    let mut tasks: Vec<_> = vec![];

    let services_map = match section.1.as_hash() {
        Some(map) => map,
        None => bail!("None service provided, aborting."),
    };
    for service in services_map.iter() {
        let service_name = service.0.as_str().unwrap().to_string();
        let service_map = match service.1.as_hash() {
            Some(map) => map,
            None => bail!("Provided service is not a valid map"),
        };
        let interval = interval(service_map)?;
        let info = TaskInfo::new(service_name, interval);
        let checker = type_data(service_map)?;

        tasks.push(Task::new(info, checker));
    }

    Ok(tasks)
}
fn interval(service_attrs: &Hash) -> anyhow::Result<SignedDuration> {
    let interval_value = match service_attrs.get(&Yaml::String("interval".into())) {
        Some(value) => value,
        None => bail!("'interval' is mandatory field for a service."),
    };
    match interval_value.as_i64() {
        Some(interval) => {
            if interval < 0 {
                bail!("Interval must be grater than 0")
            } else {
                Ok(SignedDuration::from_secs(interval))
            }
        }
        None => bail!("'interval' must be a number."),
    }
}
fn type_data(service_attrs: &Hash) -> anyhow::Result<Checker> {
    let service_config = match service_attrs.get(&Yaml::String("configuration".into())) {
        Some(config) => config,
        None => bail!("'configuration' is mandatory map field for a service."),
    };
    let config_map = match service_config.as_hash() {
        Some(map) => map,
        None => bail!("'configuration' is not valid map."),
    };

    Checker::try_from(config_map)
}
