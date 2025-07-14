use std::process::exit;

use toktok::configuration::load_config;

#[tokio::main]
async fn main() {
    let scheduler = match load_config() {
        Ok(s) => s,
        Err(err) => {
            println!("{err}");
            exit(1)
        }
    };

    scheduler.start().await
}
