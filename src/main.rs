use clap::Parser;
use toktok::{args::Args, parser::Configuration, scheduler::Scheduler};
use tracing::{Level, event};

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    let config = Configuration::builder(&args)
        .map_err(|e| format!("{e}"))?
        .services()?
        .mailer()?
        .build();

    // let config = match load_config(&args) {
    //     Ok(s) => s,
    //     Err(err) => {
    //         eprintln!("\x1b[31merror: \x1b[0mIncorrect data found in configuration\n\n{err}");
    //         exit(1);
    //     }
    // };

    if config.has_tasks() {
        return Err("None services found to monitor, shutting down".to_string());
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
    scheduler.init(notifiers).await;

    Ok(())
}
