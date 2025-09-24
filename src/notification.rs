use std::{
    fs,
    io::{self, Write},
    path::Path,
};

use anyhow::bail;
use jiff::{Zoned, civil::DateTime};
use yaml_rust2::{Yaml, yaml::Hash};

use crate::executor::ExecutionResult;

pub struct NotifierContent {
    started_at: DateTime,
    finished_at: DateTime,
    exec_resp: ExecutionResult,
}
impl NotifierContent {
    pub fn new(started_at: DateTime, finished_at: DateTime, exec_resp: ExecutionResult) -> Self {
        Self {
            started_at,
            finished_at,
            exec_resp,
        }
    }
}

#[derive(Debug)]
pub enum Notifier {
    File(FileNotifier),
}

#[derive(Debug)]
pub struct FileNotifier {
    directory: String,
}

impl FileNotifier {
    pub fn create_and_check_dir(dir: &str) -> Result<(), anyhow::Error> {
        let dir_path = Path::new(&dir);
        if !dir_path.exists() {
            match fs::create_dir_all(dir_path) {
                Ok(_) => (),
                Err(err) => bail!(
                    "Unable to create the directory for file notification: {:#?}",
                    err
                ),
            }
        }

        let filepath = format!("{}toktok-checker", dir);
        let filepath = Path::new(&filepath);
        match fs::File::create(filepath) {
            Ok(_) => {
                let _ = fs::remove_file(filepath);
            }
            Err(_) => bail!(
                "The program will not be able to write in {}, check the permissions and try again",
                &dir
            ),
        }

        Ok(())
    }

    pub fn new(directory: String) -> Self {
        Self { directory }
    }

    pub fn write(&self, content: NotifierContent) -> std::result::Result<(), io::Error> {
        let today = Zoned::now().date();
        let filename = format!("{}-{}-{}.txt", today.year(), today.month(), today.day());
        let filepath = format!("{}{}", self.directory, filename);

        let content = format!(
            "Started at: {} - Finished at: {}\nStatus: {}\nMessage: {}",
            content.started_at,
            content.finished_at,
            content.exec_resp.status,
            content.exec_resp.message
        );

        if !Path::new(&filepath).exists() {
            let _ = fs::File::create(&filepath);
        }
        let mut file = fs::File::open(&filepath).unwrap();
        file.write_all(content.as_bytes())
    }
}

impl TryFrom<&Hash> for Notifier {
    type Error = anyhow::Error;

    fn try_from(data_config: &Hash) -> Result<Self, Self::Error> {
        let notif_type = match data_config.get(&Yaml::String("type".into())) {
            Some(t) => t.as_str().unwrap().to_lowercase(),
            None => bail!("'type' is mandatory field for a notification"),
        };
        match notif_type.as_str() {
            "file" => {
                let mut directory = match data_config.get(&Yaml::String("directory".into())) {
                    Some(d) => d.as_str().unwrap().to_string(),
                    None => bail!("'directory' is a mandatory field in file type notification"),
                };
                if !directory.ends_with('/') {
                    directory.push('/');
                }
                FileNotifier::create_and_check_dir(&directory)?;

                Ok(Notifier::File(FileNotifier::new(directory)))
            }
            _ => bail!("Type '{}' is not valid", notif_type),
        }
    }
}
