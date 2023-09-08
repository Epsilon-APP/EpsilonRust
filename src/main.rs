#[macro_use]
extern crate log;

use std::io::Write;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};

use env_logger::fmt::Color;
use k8s_openapi::chrono::Local;
use kube::CustomResourceExt;
use log::Level;
use tokio::time::sleep;

use crate::config::EpsilonConfig;
use crate::context::Context;
use crate::controller::definitions::epsilon_instance::EpsilonInstance;
use crate::controller::definitions::epsilon_queue::EpsilonQueue;
use crate::controller::epsilon_controller::EpsilonController;
use crate::epsilon::api::epsilon_api::EpsilonApi;
use crate::epsilon::queue::queue_provider::QueueProvider;
use crate::epsilon::server::instances::instance_provider::InstanceProvider;
use crate::epsilon::server::instances::EResult;
use crate::epsilon::server::templates::template_provider::TemplateProvider;
use crate::tasks::hub_task::HubTask;
use crate::tasks::proxy_task::ProxyTask;
use crate::tasks::queue_task::QueueTask;
use crate::tasks::task::Task;
use crate::tasks::task_builder::TaskBuilder;

pub mod controller;

pub mod epsilon;
pub mod tasks;

pub mod config;
pub mod context;

#[tokio::main]
async fn main() -> EResult<()> {
    let path_name = "./resources";

    fs::create_dir(path_name).ok();

    fs::write(
        format!("{}/{}", path_name, "epsilon_instance-definition.yaml"),
        serde_yaml::to_string(&EpsilonInstance::crd()).unwrap(),
    )
    .unwrap();

    fs::write(
        format!("{}/{}", path_name, "epsilon_queue-definition.yaml"),
        serde_yaml::to_string(&EpsilonQueue::crd()).unwrap(),
    )
    .unwrap();

    std::env::set_var(
        "RUST_LOG",
        "epsilon=error, epsilon=warn, epsilon=info, epsilon=debug, epsilon=trace, rocket=info, rocket=error",
    );

    env_logger::Builder::new()
        .parse_default_env()
        .format(|buf, record| {
            let mut style = buf.style();

            match record.level() {
                Level::Error => style.set_color(Color::Red).set_bold(true),
                Level::Warn => style.set_color(Color::Yellow).set_bold(true),
                Level::Info => style.set_color(Color::Blue).set_bold(true),

                _ => style.set_color(Color::White).set_bold(true),
            };

            writeln!(
                buf,
                "[{} {}] {}",
                Local::now().format("%H:%M:%S"),
                style.value(record.level()),
                style.value(record.args())
            )
        })
        .init();

    let epsilon = concat!(
        "┌──────────────────────────────────────┐\n",
        "│   _____           _ _                │\n",
        "│  |  ___|        (_) |                │\n",
        "│  | |__ _ __  ___ _| | ___  _ __      │\n",
        "│  |  __| '_ \\/ __| | |/ _ \\| '_ \\     │\n",
        "│  | |__| |_) \\__ \\ | | (_) | | | |    │\n",
        "│  \\____/ .__/|___/_|_|\\___/|_| |_|    │\n",
        "│        | |                           │\n",
        "│        |_|                           │\n",
        "├──────────────────────────────────────┤\n",
        "────────────────────────────────────────\n",
    );

    println!("{}", epsilon);

    info!("Version : {}", env!("CARGO_PKG_VERSION"));

    let namespace =
        fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/namespace").unwrap();

    info!("Epsilon listen in namespace: {}", namespace);

    let config = EpsilonConfig::load("./config.json");

    let epsilon_api = EpsilonApi::new();

    let template_provider = TemplateProvider::new(&config);

    let controller = EpsilonController::new(&namespace, &template_provider).await;
    let instance_provider = InstanceProvider::new(&controller);

    let queue_provider = QueueProvider::new(&instance_provider, &template_provider).await?;

    let context = Context::new(
        epsilon_api,
        template_provider,
        instance_provider,
        queue_provider,
    );

    info!("Instance provider has been started");

    sleep(Duration::from_secs(10)).await;

    TaskBuilder::new()
        .ignite_task(ProxyTask::init(Arc::clone(&context)).await?, 6000)
        .ignite_task(HubTask::init(Arc::clone(&context)).await?, 2000)
        .ignite_task(QueueTask::init(Arc::clone(&context)).await?, 2000);

    info!("Tasks have been started");

    let figment = rocket::config::Config::figment()
        .merge(("ident", "Epsilon"))
        .merge(("address", "0.0.0.0"));

    let _ = rocket::custom(figment)
        .manage(Arc::clone(&context))
        .mount("/", rocket::routes![epsilon::api::routes::ping])
        .mount("/api", rocket::routes![epsilon::api::routes::events])
        .mount("/queue", rocket::routes![epsilon::queue::routes::push])
        .mount(
            "/instance",
            rocket::routes![
                epsilon::server::instances::routes::create,
                epsilon::server::instances::routes::close,
                epsilon::server::instances::routes::in_game,
                epsilon::server::instances::routes::get,
                epsilon::server::instances::routes::get_all,
                epsilon::server::instances::routes::get_from_template
            ],
        )
        .launch()
        .await?;

    Ok(())
}
