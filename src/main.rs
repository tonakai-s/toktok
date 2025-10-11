use std::process;

use clap::Parser;
use toktok::{
    args::Args,
    parser::{Configuration, error::ConfigError},
    scheduler::Scheduler,
};
use tracing::{Level, event};

#[tokio::main]
async fn main() -> Result<(), Box<dyn ConfigError>> {
    tracing_subscriber::fmt::init();

    if let Err(e) = entrypoint().await {
        eprint!("Error: {}", e);
        process::exit(1);
    }

    Ok(())
}

async fn entrypoint() -> Result<(), Box<dyn ConfigError>> {
    let args = Args::parse();

    let config = Configuration::builder(&args)?
        .services()?
        .mailer()?
        .build()?;

    event!(
        Level::INFO,
        temp_dir_path = std::env::temp_dir().to_str().unwrap()
    );

    let mut notifiers: Vec<_> = vec![];
    if let Some(mailer) = &config.mailer {
        notifiers.push(mailer.clone());
    }

    let scheduler = Scheduler::new(config);
    scheduler.init(notifiers).await;

    Ok(())
}
