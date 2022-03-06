#[macro_use]
extern crate log;

use crate::epsilon::api::epsilon_api::EpsilonApi;
use crate::epsilon::queue::queue_provider::QueueProvider;
use crate::epsilon::server::instance_provider::InstanceProvider;
use crate::epsilon::server::EResult;
use crate::k8s::kube::Kube;
use crate::tasks::hub_task::HubTask;
use crate::tasks::proxy_task::ProxyTask;
use crate::tasks::queue_task::QueueTask;
use crate::tasks::task::Task;
use crate::tasks::task_builder::TaskBuilder;
use env_logger::fmt::Color;
use k8s_openapi::chrono::Local;
use log::{Level, LevelFilter};
use std::io::Write;
use std::sync::Arc;

mod epsilon;
mod tasks;

mod k8s;

#[tokio::main]
async fn main() -> EResult<()> {
    std::env::set_var(
        "RUST_LOG",
        "EpsilonRust=info, EpsilonRust=error, EpsilonRust=debug",
    );

    env_logger::Builder::new()
        .format(|buf, record| {
            let mut style = buf.style();

            match record.level() {
                Level::Error => style.set_color(Color::Red).set_bold(true),

                Level::Warn => style.set_color(Color::Yellow).set_bold(true),

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
        .filter(None, LevelFilter::Info)
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
        "│        Started Version: {}        │\n",
        "└──────────────────────────────────────┘\n"
    );

    println!("{}", epsilon.replace("{}", env!("CARGO_PKG_VERSION")));

    let kube = Kube::new("epsilon").await;

    info!(
        "Kube client has been started (Namespace={}, Version={})",
        kube.get_namespace(),
        kube.get_info().git_version
    );

    let epsilon_api = EpsilonApi::new();
    let instance_provider = InstanceProvider::new(&kube);
    let queue_provider = QueueProvider::new(&instance_provider).await?;

    info!("Instance provider has been started");

    TaskBuilder::new(&instance_provider)
        .ignite_task(
            ProxyTask::init(&epsilon_api, &instance_provider, &queue_provider).await?,
            6000,
        )
        .ignite_task(
            HubTask::init(&epsilon_api, &instance_provider, &queue_provider).await?,
            2000,
        )
        .ignite_task(
            QueueTask::init(&epsilon_api, &instance_provider, &queue_provider).await?,
            2000,
        );

    info!("Tasks have been started");

    let figment = rocket::config::Config::figment().merge(("address", "0.0.0.0"));

    rocket::custom(figment)
        .manage(Arc::clone(&epsilon_api))
        .manage(Arc::clone(&instance_provider))
        .manage(Arc::clone(&queue_provider))
        .mount("/api", rocket::routes![epsilon::api::epsilon_api::events])
        .mount(
            "/instance",
            rocket::routes![epsilon::server::instance_provider::create],
        )
        .mount(
            "/instance",
            rocket::routes![epsilon::server::instance_provider::close],
        )
        .mount(
            "/instance",
            rocket::routes![epsilon::server::instance_provider::in_game],
        )
        .mount(
            "/instance",
            rocket::routes![epsilon::server::instance_provider::get],
        )
        .mount(
            "/instance",
            rocket::routes![epsilon::server::instance_provider::get_all],
        )
        .mount(
            "/queue",
            rocket::routes![epsilon::queue::queue_provider::push],
        )
        .launch()
        .await?;

    Ok(())
}
