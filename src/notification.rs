use std::{
    fmt::Display,
    fs::File,
    io::{self, Write},
    sync::{Arc, Mutex},
};

use jiff::{Zoned, civil::DateTime};

type FileNotify = Arc<Mutex<File>>;

enum ServiceType {
    Web,
}

impl Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceType::Web => write!(f, "Web"),
        }
    }
}

pub enum Notifier {
    File(FileNotifier),
}

pub struct FileNotifier {
    file: FileNotify,
    service_type: ServiceType,
    service_name: String,
    datetime: DateTime,
}

impl FileNotifier {
    pub fn new(
        file: FileNotify,
        service_type: ServiceType,
        service_name: String,
    ) -> Self {
        Self {
            file,
            service_type,
            service_name,
            datetime: Zoned::now().datetime(),
        }
    }

    pub fn write(&self, message: &str) -> std::result::Result<(), io::Error> {
        let content = format!(
            "{} - Service: {} - Type {}: {}",
            self.datetime.to_string(),
            self.service_name,
            self.service_type,
            message
        );
        self.file.lock().unwrap().write_all(content.as_bytes())
    }
}
