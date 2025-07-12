use jiff::SignedDuration;
use reqwest::StatusCode;
use toktok::{
    data::{Data, WebData},
    scheduler::Scheduler,
    task::Task,
};

#[tokio::main]
async fn main() {
    let mut scheduler = Scheduler::new();

    let data = Data::Web(WebData::new(
        String::from("https://tuamaeaquelaursa.com"),
        None,
        StatusCode::from_u16(200).unwrap(),
    ));
    let task = Task::new(
        String::from("site.tuamaeaquelaursa10secs"),
        SignedDuration::from_secs(10),
    );
    scheduler.enqueue(task, data);

    scheduler.start().await
}
