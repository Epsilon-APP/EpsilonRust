use crate::epsilon::epsilon_error::EpsilonError;
use crate::epsilon::server::instances::common::instance_type::InstanceType;
use crate::Context;
use rocket::State;
use serde_json::json;
use std::sync::Arc;

#[rocket::post("/create/<template>")]
pub async fn create(template: &str, context: &State<Arc<Context>>) {
    let instance_provider = context.get_instance_provider();

    instance_provider
        .start_instance(template)
        .await
        .map_err(|_| {
            EpsilonError::ApiServerError(format!(
                "Failed to create an instance from template ({})",
                template
            ))
        })
        .unwrap();

    info!("An instance has been created (template={})", template);
}

#[rocket::post("/close/<instance>")]
pub async fn close(instance: &str, context: &State<Arc<Context>>) {
    let instance_provider = context.get_instance_provider();

    instance_provider
        .remove_instance(instance)
        .await
        .map_err(|_| {
            EpsilonError::ApiServerError(format!("Failed to close instance ({})", instance))
        })
        .unwrap();
}

#[rocket::post("/in_game/<instance>")]
pub async fn in_game(instance: &str, context: &State<Arc<Context>>) {
    let instance_provider = context.get_instance_provider();

    instance_provider
        .set_in_game_instance(instance, true)
        .await
        .map_err(|_| {
            EpsilonError::ApiServerError(format!("Failed to set in game instance ({})", instance))
        })
        .unwrap();

    info!("An instance is now in game (name={})", instance);
}

#[rocket::get("/get/<template>")]
pub async fn get(template: &str, context: &State<Arc<Context>>) -> String {
    // let instance_provider = context.get_instance_provider();
    //
    // let instances = instance_provider
    //     .get_instances(&InstanceType::Server, Some(template), None, false)
    //     .await
    //     .map_err(|_| {
    //         EpsilonError::ApiServerError(format!(
    //             "Failed to get instance from template {}",
    //             template
    //         ))
    //     })
    //     .unwrap()
    //     .into_iter();
    //
    // let mut json_array: Vec<InstanceJson> = Vec::with_capacity(instances.len());
    //
    // for instance in instances {
    //     json_array.push(instance.to_json().await);
    // }
    //
    // json!({ "instances": json_array }).to_string()
    String::from("test")
}

#[rocket::get("/get_all")]
pub async fn get_all(context: &State<Arc<Context>>) -> String {
    // let instance_provider = context.get_instance_provider();
    //
    // let instances = instance_provider
    //     .get_instances(&InstanceType::Server, None, None, false)
    //     .await
    //     .map_err(|_| EpsilonError::ApiServerError("Failed to get every instance".to_string()))
    //     .unwrap()
    //     .into_iter();
    //
    // let mut json_array: Vec<InstanceJson> = Vec::new();
    //
    // for instance in instances {
    //     let json = instance.to_json().await;
    //     json_array.push(json);
    // }
    //
    // json!({ "instances": json_array }).to_string()
    String::from("test")
}

#[rocket::get("/get_from_name/<instance_name>")]
pub async fn get_from_name(instance_name: &str, context: &State<Arc<Context>>) -> String {
    // let instance_provider = context.get_instance_provider();
    //
    // let instance = instance_provider.get_instance(instance_name).await.unwrap();
    //
    // serde_json::to_string(&instance.to_json().await).unwrap()
    String::from("test")
}
