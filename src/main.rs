use std::process::exit;

use clap::Parser;
use toktok::{args::Args, configuration::load_config, scheduler::Scheduler};
use tracing::{Level, event};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let config = match load_config(&args) {
        Ok(s) => s,
        Err(err) => {
            eprintln!("The program found an error during the configuration parse:\n{err}");
            exit(1);
        }
    };
    if config.tasks.is_empty() {
        println!("None services found to monitor, shutting down");
        exit(0);
    }

    event!(
        Level::INFO,
        temp_dir_path = std::env::temp_dir().to_str().unwrap()
    );

    let mut notifiers: Vec<_> = vec![];
    if let Some(mailer) = &config.mailer {
        notifiers.push(mailer.clone());
    }

    let scheduler = Scheduler::new(config);
    scheduler.start(notifiers).await
}
