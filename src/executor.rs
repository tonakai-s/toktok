use std::sync::mpsc::Sender;

use tracing::{Level, event};

use crate::{
    checker::{
        Checker,
        structs::{CheckerResult, CheckerStatus},
    },
    task::Task,
};

/// This function calls the checker of the received function.
/// Validate the result, if necessary send the result to the notifier thread.
/// Finally it sents the task back to the enqueuer thread.
pub async fn execute_check(
    mut task: Task,
    tx_task: Sender<Task>,
    tx_notifier: Sender<CheckerResult>,
) {
    task.set_last_execution_at();
    let checker_result = match task.checker() {
        Checker::Web(checker) => checker.check(&task.name()).await,
        Checker::Server(checker) => checker.check(&task.name()).await,
    };
    task.log(&checker_result);
    if checker_result.status != CheckerStatus::Success
        && let Err(err) = tx_notifier.send(checker_result)
    {
        event!(
            Level::ERROR,
            message = ?err.0,
            error = %err,
            "Error sending the checker result to notifier thread"
        );
    }
    if let Err(err) = tx_task.send(task) {
        event!(
            Level::ERROR,
            message = ?err.0,
            error = %err,
            "Error sending task to the enqueuer thread"
        );
    }
}
