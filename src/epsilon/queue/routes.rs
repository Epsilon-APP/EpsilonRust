use std::sync::Arc;

use rocket::serde::json::Json;
use rocket::State;

use crate::epsilon::epsilon_error::EpsilonError;
use crate::epsilon::queue::common::group::Group;
use crate::Context;

#[rocket::post("/push", data = "<body>")]
pub async fn push(body: Json<Group>, context: &State<Arc<Context>>) -> Result<(), EpsilonError> {
    let queue_provider = context.get_queue_provider();

    let queue_name = &body.queue;
    let queue_map = queue_provider.get_queues();

    let mut queue = queue_map
        .get(queue_name)
        .ok_or(EpsilonError::QueueNotFoundError(queue_name.to_owned()))?
        .write()
        .await;

    info!(
        "Player {} added to queue {}",
        &body.players.join("/"),
        &body.queue
    );

    queue.push(body.into_inner());

    Ok(())
}
