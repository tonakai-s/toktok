use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{Result, bail};
use jiff::{Zoned, civil::Date};
use tracing::{Level, event, span};

use crate::checker::structs::CheckerResult;

#[derive(Debug)]
pub struct TaskLogger {
    filename: String,
    file_dir: PathBuf,
    file_date: Date,
    file: File,
}

impl TaskLogger {
    pub fn new(task_name: &str) -> Result<Self> {
        let span = span!(Level::TRACE, "task_logger::new");
        let _enter = span.enter();

        let filename = TaskLogger::todays_filename(&task_name);

        let log_path = TaskLogger::folder(&task_name);
        if !log_path.exists() {
            match fs::create_dir_all(&log_path) {
                Ok(_) => (),
                Err(err) => bail!(
                    "The program was unable to create the logs directory. Error: {}",
                    err
                ),
            }
        }

        let full_log_filepath = format!("{}{}", log_path.to_str().unwrap(), filename);
        let full_log_filepath = Path::new(&full_log_filepath);
        if !full_log_filepath.exists() {
            if let Err(err) = fs::File::create(&full_log_filepath) {
                bail!(
                    "The program was unable to create the logs file. Error: {}",
                    err
                );
            }
        }

        let file = match fs::OpenOptions::new()
            .write(true)
            .append(true)
            .open(&full_log_filepath)
        {
            Ok(file) => file,
            Err(err) => {
                bail!(
                    "The program was unable to open the logs file. Error: {}",
                    err
                );
            }
        };
        Ok(Self {
            filename,
            file_dir: log_path,
            file_date: Zoned::now().date(),
            file,
        })
    }

    fn folder(task_name: &str) -> PathBuf {
        let tempdir = std::env::temp_dir();
        if tempdir.ends_with("/") {
            let file_path = format!(
                "{}{}/{}/",
                tempdir.to_str().unwrap(),
                "toktok",
                task_name,
            );
            PathBuf::from(file_path)
        } else {
            let file_path = format!(
                "{}/{}/{}/",
                tempdir.to_str().unwrap(),
                "toktok",
                task_name,
            );
            PathBuf::from(file_path)
        }
    }

    fn todays_filename(task_name: &str) -> String {
        let today = Zoned::now().date();
        format!(
            "{}-{}-{}-{}.log",
            today.year(),
            today.month(),
            today.day(),
            task_name
        )
    }

    pub fn log(&mut self, execution_result: &CheckerResult) {
        let span = span!(Level::TRACE, "task_logger::log");
        let _enter = span.enter();

        if self.file_date != Zoned::now().date() {
            self.update_file();
        }

        let content = format!(
            "[{}] {} - {}\n",
            Zoned::now().datetime().to_string(),
            execution_result.status,
            execution_result.message,
        );
        if let Err(err) = self.file.write_all(content.as_bytes()) {
            event!(
                Level::ERROR,
                error = %err,
                "Error writing the task result to general log"
            );
        }
    }

    fn update_file(&mut self) {
        let span = span!(Level::TRACE, "task_logger::update_file");
        let _enter = span.enter();

        let full_log_filepath = format!(
            "{}/{}",
            self.file_dir.to_str().unwrap(),
            TaskLogger::todays_filename(&self.filename)
        );
        let full_log_filepath = Path::new(&full_log_filepath);
        if !full_log_filepath.exists() {
            if let Err(err) = fs::File::create(&full_log_filepath) {
                event!(
                    Level::ERROR,
                    error = %err,
                    "Error updating the general task log file"
                );
                panic!(
                    "Error updating the general task log file. For more info check the tracing file."
                );
            }
        }

        let file = match fs::File::open(&full_log_filepath) {
            Ok(file) => file,
            Err(err) => {
                event!(
                    Level::ERROR,
                    error = %err,
                    "Error opening the general task log file"
                );
                panic!(
                    "Error opening the general task log file. For more info check the tracing file."
                );
            }
        };

        self.file = file;
    }
}
