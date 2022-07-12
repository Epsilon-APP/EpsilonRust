use crate::epsilon::queue::common::group::Group;
use crate::Context;
use rocket::serde::json::Json;
use rocket::State;
use std::sync::Arc;

#[rocket::post("/push", data = "<body>")]
pub async fn push(body: Json<Group>, context: &State<Arc<Context>>) {
    let queue_provider = context.get_queue_provider();

    let queue_map = queue_provider.get_queues();
    let mut queue = queue_map.get(&body.queue).unwrap().write().await;

    info!(
        "Player {} added to queue {}",
        &body.players.join("/"),
        &body.queue
    );

    queue.push(body.into_inner());
}
