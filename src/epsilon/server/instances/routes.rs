use std::sync::Arc;

use rocket::State;
use serde_json::json;

use crate::controller::definitions::epsilon_instance::InstanceJson;
use crate::epsilon::epsilon_error::EpsilonError;
use crate::epsilon::server::instances::common::instance_type::InstanceType;
use crate::Context;

#[rocket::post("/create/<template>")]
pub async fn create(template: &str, context: &State<Arc<Context>>) -> Result<(), EpsilonError> {
    let instance_provider = context.get_instance_provider();

    instance_provider
        .start_instance(template)
        .await
        .map_err(|_| {
            EpsilonError::ApiServerError(format!(
                "Failed to create an instance from template ({})",
                template
            ))
        })?;

    info!("An instance has been created (template={})", template);

    Ok(())
}

#[rocket::post("/close/<instance>")]
pub async fn close(instance: &str, context: &State<Arc<Context>>) -> Result<(), EpsilonError> {
    let instance_provider = context.get_instance_provider();

    instance_provider
        .remove_instance(instance)
        .await
        .map_err(|_| {
            EpsilonError::ApiServerError(format!("Failed to close instance ({})", instance))
        })?;

    info!("An instance has been closed (instance={})", instance);

    Ok(())
}

#[rocket::post("/in_game/<instance>")]
pub async fn in_game(instance: &str, context: &State<Arc<Context>>) -> Result<(), EpsilonError> {
    let instance_provider = context.get_instance_provider();

    instance_provider
        .enable_in_game_instance(instance)
        .await
        .map_err(|_| {
            EpsilonError::ApiServerError(format!("Failed to set in game instance ({})", instance))
        })?;

    info!("An instance is now in game (name={})", instance);

    Ok(())
}

#[rocket::get("/get/<instance_name>")]
pub async fn get(
    instance_name: &str,
    context: &State<Arc<Context>>,
) -> Result<String, EpsilonError> {
    let instance_provider = context.get_instance_provider();
    let instance = instance_provider.get_instance(instance_name).await?;

    Ok(serde_json::to_string(&instance.to_json().await?)
        .map_err(|_| EpsilonError::ParseJsonError)?)
}

#[rocket::get("/get_all")]
pub async fn get_all(context: &State<Arc<Context>>) -> Result<String, EpsilonError> {
    let instance_provider = context.get_instance_provider();

    let instances = instance_provider
        .get_instances(InstanceType::Server, None, None)
        .await
        .map_err(|_| EpsilonError::ApiServerError("Failed to get every instance".to_string()))?
        .into_iter();

    let mut json_array: Vec<InstanceJson> = Vec::new();

    for instance in instances {
        let json = instance.to_json().await?;
        json_array.push(json);
    }

    Ok(json!({ "instances": json_array }).to_string())
}

#[rocket::get("/get_from_template/<template>")]
pub async fn get_from_template(
    template: &str,
    context: &State<Arc<Context>>,
) -> Result<String, EpsilonError> {
    let instance_provider = context.get_instance_provider();

    let instances = instance_provider
        .get_instances(InstanceType::Server, Some(template), None)
        .await
        .map_err(|_| {
            EpsilonError::ApiServerError(format!(
                "Failed to get instance from template {}",
                template
            ))
        })?
        .into_iter();

    let mut json_array: Vec<InstanceJson> = Vec::with_capacity(instances.len());

    for instance in instances {
        let json = instance.to_json().await?;
        json_array.push(json);
    }

    Ok(json!({ "instances": json_array }).to_string())
}
