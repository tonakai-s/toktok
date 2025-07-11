use jiff::{SignedDuration, civil::DateTime};
use reqwest::StatusCode;
use toktok::{scheduler::Scheduler, task::{ApplicationType, Task, TaskStatus, WebApplicationType}};

#[tokio::main]
async fn main() {
    let mut scheduler = Scheduler::new();

    let application = WebApplicationType::new(
        String::from("https://tuamaeaquelaursa.com"),
        None,
        StatusCode::from_u16(200).unwrap(),
    );
    let task = Task::new(
        SignedDuration::from_secs(10),
        SignedDuration::from_secs(10),
        DateTime::default(),
        ApplicationType::Web(application),
        TaskStatus::Waiting,
    );
    scheduler.enqueue(task);

    scheduler.start().await
}
