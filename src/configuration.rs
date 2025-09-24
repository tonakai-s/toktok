use std::io::Read;

use anyhow::bail;
use jiff::SignedDuration;
use yaml_rust2::{Yaml, YamlLoader, yaml::Hash};

use crate::{
    checker::Checker, notification::Notifier, scheduler::Scheduler, task::Task, task_info::TaskInfo,
};

pub fn load_config() -> anyhow::Result<Scheduler> {
    let mut content = String::new();
    let mut file = match std::fs::File::open("services.yaml") {
        Ok(f) => f,
        Err(err) => bail!("Error trying to open the config file: {:#?}", err),
    };
    match file.read_to_string(&mut content) {
        Ok(_) => {}
        Err(err) => bail!("Error trying to read the config file: {:#?}", err),
    };
    let mut config = match YamlLoader::load_from_str(&content) {
        Ok(c) => c,
        Err(err) => bail!(
            "Error interpreting basic structure of config file: {:#?}",
            err
        ),
    };

    parse_config(&mut config)
}

fn parse_config(config: &mut [Yaml]) -> anyhow::Result<Scheduler> {
    let mut scheduler: Option<_> = None;
    for section in config[0].as_hash().unwrap().iter() {
        if section.0.as_str().unwrap() == "services" {
            scheduler = Some(parse_services(&section)?);
        }
    }

    // Have another way to do this, need look at hello-crate repo
    Ok(scheduler.unwrap())
}

fn parse_services(section: &(&Yaml, &Yaml)) -> anyhow::Result<Scheduler> {
    let mut scheduler = Scheduler::default();

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
        let notifier = notification(service_map)?;

        scheduler.enqueue(Task::new(info, checker, notifier));
    }

    Ok(scheduler)
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
fn notification(service_attrs: &Hash) -> anyhow::Result<Notifier> {
    let service_notification = match service_attrs.get(&Yaml::String("notification".into())) {
        Some(notification) => notification,
        None => bail!("'notification' is mandatory map field for a service."),
    };
    let notification_map = match service_notification.as_hash() {
        Some(map) => map,
        None => bail!("'notification' is not valid map."),
    };

    Notifier::try_from(notification_map)
}
