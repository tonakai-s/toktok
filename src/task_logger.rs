use std::{fs::{self, File}, io::Write, path::{Path, PathBuf}};

use anyhow::{bail, Result};
use jiff::{civil::Date, Zoned};

use crate::executor::ExecutionResult;

#[derive(Debug)]
pub struct TaskLogger {
    filename: String,
    file_dir: PathBuf,
    file_date: Date,
    file: File
}

impl TaskLogger {
    pub fn new(filename: String, dir: Option<&str>) -> Result<Self> {
        let filename = TaskLogger::todays_filename(&filename);
        let mut log_dir = "/tmp/toktok/";
        if let Some(dir) = dir {
            log_dir = dir.trim_end_matches('/');
        }

        let log_path = PathBuf::from(log_dir);
        if !log_path.exists() {
            match fs::create_dir_all(&log_path) {
                Ok(_) => (),
                Err(err) => bail!("The program was unable to create the logs directory. Error: {}", err),
            }
        }

        let full_log_filepath = format!(
            "{}{}",
            log_dir,
            filename
        );
        let full_log_filepath = Path::new(&full_log_filepath);
        if !full_log_filepath.exists() {
            let _ = fs::File::create(&full_log_filepath);
        }

        let file = fs::OpenOptions::new().write(true).append(true).open(&full_log_filepath).unwrap();
        Ok(Self {
            filename,
            file_dir: log_path,
            file_date: Zoned::now().date(),
            file,
        })
    }

    fn todays_filename(filename: &str) -> String {
        let today = Zoned::now().date();
        format!(
            "{}-{}-{}-{}.log",
            today.year(),
            today.month(),
            today.day(),
            filename
        )
    }

    pub fn log(&mut self, execution_result: ExecutionResult) {
        if self.file_date != Zoned::now().date() {
            self.update_file();
        }

        let content = format!(
            "[{}] {} - {}\n",
            Zoned::now().datetime().to_string(),
            execution_result.status,
            execution_result.message,
            
        );
        let _ = self.file.write_all(content.as_bytes());
    }

    fn update_file(&mut self) {
        let full_log_filepath = format!(
            "{}/{}",
            self.file_dir.to_str().unwrap(),
            TaskLogger::todays_filename(&self.filename)
        );
        let full_log_filepath = Path::new(&full_log_filepath);
        if !full_log_filepath.exists() {
            let _ = fs::File::create(&full_log_filepath);
        }

        let file = fs::File::open(&full_log_filepath).unwrap();
        self.file = file;
    }
}