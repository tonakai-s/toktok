use std::{
    fmt::Display,
    io::{self, Read},
};

use jiff::SignedDuration;
use yaml_rust2::{ScanError, Yaml, YamlLoader};

use crate::{
    args::Args,
    checker::{Checker, structs::CheckerParserError},
    notification::email::MailNotifier,
    parser::ConfigKey,
    task::Task,
    task_info::TaskInfo,
};

const DEFAULT_CONFIG_FILE: &str = "toktok.yaml";

#[derive(Debug)]
pub enum ConfigFileError {
    UnableToOpen(io::Error),
    UnableToRead(io::Error),
    UnableToScan(ScanError),
}
impl Display for ConfigFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigFileError::UnableToOpen(err) => {
                write!(f, "Error trying to open the config file: {err}")
            }
            ConfigFileError::UnableToRead(err) => {
                write!(f, "Error trying to read the config file: {err}")
            }
            ConfigFileError::UnableToScan(err) => write!(
                f,
                "Error interpreting the basic structure of config file: {err}"
            ),
        }
    }
}

#[derive(Debug)]
pub enum ConfigParseError {
    NoServiceProvided,
}
impl Display for ConfigParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigParseError::NoServiceProvided => {
                write!(f, "None service provided, aborting.")
            }
        }
    }
}

#[derive(Default)]
pub struct Configuration {
    pub tasks: Vec<Task>,
    pub mailer: Option<MailNotifier>,
}

pub fn load_config(args: &Args) -> Result<Configuration, String> {
    let mut content = String::new();
    let config_path = args
        .config
        .as_ref()
        .map_or(DEFAULT_CONFIG_FILE, |config| config);

    let mut file = match std::fs::File::open(config_path) {
        Ok(f) => f,
        Err(err) => return Err(format!("{}", ConfigFileError::UnableToOpen(err))),
    };
    match file.read_to_string(&mut content) {
        Ok(_) => {}
        Err(err) => return Err(format!("{}", ConfigFileError::UnableToRead(err))),
    };
    let config = match YamlLoader::load_from_str(&content) {
        Ok(c) => c,
        Err(err) => return Err(format!("{}", ConfigFileError::UnableToScan(err))),
    };

    parse_config(&config)
}

fn parse_config(config: &[Yaml]) -> Result<Configuration, String> {
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

fn parse_notifications(yaml: &Yaml, config: &mut Configuration) -> Result<(), String> {
    config.mailer = parse_mailer(&yaml["mailer"])?;
    Ok(())
}

fn parse_mailer(mailer: &Yaml) -> Result<Option<MailNotifier>, String> {
    if mailer.is_badvalue() {
        return Ok(None);
    }

    Ok(Some(MailNotifier::try_from(mailer)?))
}

fn parse_services(section: &(&Yaml, &Yaml)) -> Result<Vec<Task>, String> {
    let mut tasks: Vec<_> = vec![];

    let services_map = match section.1.as_hash() {
        Some(map) => map,
        None => return Err(format!("{}", ConfigParseError::NoServiceProvided)),
    };
    for service in services_map.iter() {
        let service_name = service.0.as_str().unwrap().to_string();

        let interval = interval(service.1)
            .map_err(|e| format!("{e}\nThrowed when reading service: {service_name}"))?;
        let checker = get_checker(service.1)
            .map_err(|e| format!("{e}\nThrowed when reading service: {service_name}"))?;
        let info = TaskInfo::new(service_name, interval);

        tasks.push(Task::new(info, checker));
    }

    Ok(tasks)
}
fn interval(service_attrs: &Yaml) -> Result<SignedDuration, String> {
    match &service_attrs["interval"] {
        Yaml::Integer(inter) if *inter > 0 => Ok(SignedDuration::from_secs(*inter)),
        _ => Err(String::from(
            "'interval' is a mandatory map field for a service.",
        )),
    }
}
fn get_checker(service_attrs: &Yaml) -> Result<Checker, CheckerParserError> {
    let service_config = &service_attrs["configuration"];
    if service_config.is_badvalue() {
        return Err(CheckerParserError::KeyNotFound(ConfigKey::Configuration));
    }

    Checker::try_from(service_config)
}
