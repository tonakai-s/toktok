use std::process::exit;

use toktok::{configuration::load_config, scheduler::Scheduler};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = match load_config() {
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

    let mut notifiers: Vec<_> = vec![];
    if let Some(mailer) = &config.mailer {
        notifiers.push(mailer.clone());
    }

    let scheduler = Scheduler::new(config);
    scheduler.start(notifiers).await
}
