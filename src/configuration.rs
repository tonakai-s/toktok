use std::{
    fmt::Display,
    io::{self, Read},
};

use anyhow::bail;
use jiff::SignedDuration;
use yaml_rust2::{ScanError, Yaml, YamlLoader};

use crate::{args::Args, checker::Checker, notification::email::MailNotifier, task::Task, task_info::TaskInfo};

const DEFAULT_CONFIG_FILE: &str = "toktok.yaml";

#[derive(Debug)]
pub enum ConfigurationFileError {
    UnableToOpen(io::Error),
    UnableToRead(io::Error),
    UnableToScan(ScanError),
}
impl Display for ConfigurationFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigurationFileError::UnableToOpen(err) => {
                write!(f, "Error trying to open the config file: {err}")
            }
            ConfigurationFileError::UnableToRead(err) => {
                write!(f, "Error trying to read the config file: {err}")
            }
            ConfigurationFileError::UnableToScan(err) => write!(
                f,
                "Error interpreting the basic structure of config file: {err}"
            ),
        }
    }
}

type Key = &'static str;
type AtWhy = &'static str;
type Service = String;
#[derive(Debug)]
pub enum ConfigurationParseError {
    KeyNotFound(Key, AtWhy),
    NoServiceProvided,
    InvalidServiceMap(Service),
}
impl Display for ConfigurationParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigurationParseError::KeyNotFound(key, at_why) => {
                write!(f, "Mandatory key not found: {key} {at_why}")
            }
            ConfigurationParseError::NoServiceProvided => {
                write!(f, "None service provided, aborting.")
            }
            ConfigurationParseError::InvalidServiceMap(key) => {
                write!(f, "A key has a invalid map: {key}")
            }
        }
    }
}

#[derive(Default)]
pub struct Configuration {
    pub tasks: Vec<Task>,
    pub mailer: Option<MailNotifier>,
}

pub fn load_config(args: &Args) -> anyhow::Result<Configuration> {
    let mut content = String::new();
    let config_path = args.config.as_ref().map_or(DEFAULT_CONFIG_FILE, |config| &config);

    let mut file = match std::fs::File::open(config_path) {
        Ok(f) => f,
        Err(err) => bail!(ConfigurationFileError::UnableToOpen(err)),
    };
    match file.read_to_string(&mut content) {
        Ok(_) => {}
        Err(err) => bail!(ConfigurationFileError::UnableToRead(err)),
    };
    let config = match YamlLoader::load_from_str(&content) {
        Ok(c) => c,
        Err(err) => bail!(ConfigurationFileError::UnableToScan(err)),
    };

    parse_config(&config)
}

fn parse_config(config: &[Yaml]) -> anyhow::Result<Configuration> {
    let mut configuration = Configuration::default();
    for section in config[0].as_hash().unwrap().iter() {
        if section.0.as_str().unwrap() == "services" {
            configuration.tasks = parse_services(&section)?;
        }
        if section.0.as_str().unwrap() == "notification" {
            parse_notifications(section.1, &mut configuration)?;
        }
    }

    Ok(configuration)
}

fn parse_notifications(yaml: &Yaml, config: &mut Configuration) -> anyhow::Result<()> {
    config.mailer = parse_mailer(&yaml["mailer"])?;
    Ok(())
}

fn parse_mailer(mailer: &Yaml) -> anyhow::Result<Option<MailNotifier>> {
    if mailer.is_badvalue() {
        return Ok(None);
    }

    Ok(Some(MailNotifier::try_from(mailer)?))
}

fn parse_services(section: &(&Yaml, &Yaml)) -> anyhow::Result<Vec<Task>> {
    let mut tasks: Vec<_> = vec![];

    let services_map = match section.1.as_hash() {
        Some(map) => map,
        None => bail!(ConfigurationParseError::NoServiceProvided),
    };
    for service in services_map.iter() {
        let service_name = service.0.as_str().unwrap().to_string();

        let interval = interval(service.1)?;
        let info = TaskInfo::new(service_name, interval);
        let checker = get_checker(service.1)?;

        tasks.push(Task::new(info, checker));
    }

    Ok(tasks)
}
fn interval(service_attrs: &Yaml) -> anyhow::Result<SignedDuration> {
    let interval_value = &service_attrs["interval"];
    if interval_value.is_badvalue() {
        bail!("'interval' is mandatory field for a service.");
    }

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
fn get_checker(service_attrs: &Yaml) -> anyhow::Result<Checker> {
    let service_config = &service_attrs["configuration"];
    if service_config.is_badvalue() {
        bail!("'configuration' is mandatory map field for a service.");
    }
    if service_config.as_hash().is_none() {
        bail!("'configuration' is not valid map.");
    };

    Checker::try_from(service_config)
}
