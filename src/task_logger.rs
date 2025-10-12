use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use jiff::{Zoned, civil::Date};
use tracing::{Level, event, span};

use crate::checker::structs::CheckerResult;

/// This struct holds data to generate logs for all executions of a specific thread.
#[derive(Debug)]
pub struct TaskLogger {
    task_name: String,
    file_dir: PathBuf,
    file_date: Date,
    file: File,
}

impl TaskLogger {
    pub fn try_new(task_name: &str) -> Result<Self, String> {
        let span = span!(Level::TRACE, "task_logger::new");
        let _enter = span.enter();

        let filename = TaskLogger::todays_filename(task_name);

        let log_path = TaskLogger::folder(task_name);
        if !log_path.exists() {
            fs::create_dir_all(&log_path).map_err(|e| {
                format!("The program was unable to create the logs directory. Error: {e}")
            })?;
        }

        let full_log_filepath = format!("{}{}", log_path.to_str().unwrap(), filename);
        let full_log_filepath = Path::new(&full_log_filepath);
        if !full_log_filepath.exists() {
            let _ = fs::File::create(full_log_filepath).map_err(|e| {
                format!("The program was unable to create the logs file. Error: {e}")
            })?;
        }

        let file = fs::OpenOptions::new()
            .append(true)
            .open(full_log_filepath)
            .map_err(|e| format!("The program was unable to open the logs file. Error: {e}"))?;

        Ok(Self {
            task_name: task_name.to_string(),
            file_dir: log_path,
            file_date: Zoned::now().date(),
            file,
        })
    }

    fn folder(task_name: &str) -> PathBuf {
        let tempdir = std::env::temp_dir();
        if tempdir.ends_with("/") {
            let file_path = format!("{}{}/{}/", tempdir.to_str().unwrap(), "toktok", task_name,);
            PathBuf::from(file_path)
        } else {
            let file_path = format!("{}/{}/{}/", tempdir.to_str().unwrap(), "toktok", task_name,);
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

    /// Each call validate if the current date is the same of the current file, if not it generates a new one.
    pub fn log(&mut self, execution_result: &CheckerResult) {
        let span = span!(Level::TRACE, "task_logger::log");
        let _enter = span.enter();

        if self.file_date != Zoned::now().date() {
            self.update_file();
        }

        let content = format!(
            "[{}] {} - {}\n",
            Zoned::now().datetime(),
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
            TaskLogger::todays_filename(&self.task_name)
        );
        let full_log_filepath = Path::new(&full_log_filepath);
        if !full_log_filepath.exists()
            && let Err(err) = fs::File::create(full_log_filepath)
        {
            event!(
                Level::ERROR,
                error = %err,
                "Error updating the general task log file"
            );
            panic!(
                "Error updating the general task log file. For more info check the tracing file."
            );
        }

        let file = match fs::File::open(full_log_filepath) {
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
