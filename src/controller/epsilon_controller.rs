use crate::controller::context::Context;
use crate::controller::definitions::epsilon_instance::{
    EpsilonInstance, EpsilonInstanceSpec, EpsilonInstanceStatus,
};
use crate::controller::definitions::epsilon_queue::EpsilonQueue;
use crate::epsilon::server::instances::common::state::EpsilonState;
use crate::{EResult, TemplateProvider};
use futures::stream::StreamExt;
use k8s_openapi::api::core::v1::{
    ConfigMapEnvSource, Container, EnvFromSource, ExecAction, Pod, PodSpec, Probe,
    ResourceRequirements,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::{ObjectMeta, OwnerReference};
use kube::api::{DeleteParams, ListParams, Patch, PatchParams, PostParams};
use kube::runtime::controller::Action;
use kube::runtime::controller::Error::ObjectNotFound;
use kube::runtime::finalizer::Event;
use kube::runtime::reflector::Store;
use kube::runtime::{finalizer, reflector, watcher, Controller};
use kube::ResourceExt;
use kube::{Api, Client, Config};
use kube::{Error, Resource};
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::sync::Arc;
use std::time::Duration;

pub struct EpsilonController {
    context: Arc<Context>,
    store: Arc<Store<EpsilonInstance>>,
}

impl EpsilonController {
    pub async fn new(
        namespace: &str,
        instance_provider: &Arc<TemplateProvider>,
    ) -> Arc<EpsilonController> {
        let mut config = Config::infer().await.expect("Failed to load kube config");
        let client = Client::try_from(config).expect("Failed to create kube client");

        let pod_api: Api<Pod> = Api::namespaced(client.clone(), namespace);

        let epsilon_instance_api: Api<EpsilonInstance> = Api::namespaced(client.clone(), namespace);
        let epsilon_queue_api: Api<EpsilonQueue> = Api::namespaced(client.clone(), namespace);

        let context: Arc<Context> = Arc::new(Context::new(
            pod_api.clone(),
            epsilon_instance_api.clone(),
            instance_provider,
        ));

        let clone_context = Arc::clone(&context);

        let controller = Controller::new(epsilon_instance_api, ListParams::default());
        let store = controller.store();

        tokio::spawn(async move {
            controller
                .owns(pod_api.clone(), ListParams::default())
                .run(Self::reconcile, Self::on_error, clone_context)
                .for_each(|res| async move {
                    match res {
                        Ok(o) => {}
                        Err(e) => debug!("Reconcile failed: {:?}", e),
                    }
                })
                .await;
        });

        Arc::new(EpsilonController {
            context,
            store: Arc::new(store),
        })
    }

    async fn reconcile(
        epsilon_instance: Arc<EpsilonInstance>,
        context: Arc<Context>,
    ) -> Result<Action, Error> {
        let pod_api = &context.pod_api;
        let epsilon_instance_api = &context.epsilon_instance_api;

        let template_provider = &context.template_provider;

        let mut instance_owner_reference = epsilon_instance.controller_owner_ref(&()).unwrap();
        instance_owner_reference.block_owner_deletion = Some(true);

        let instance_metadata = &epsilon_instance.metadata;
        let instance_spec = &epsilon_instance.spec;
        let instance_status = &epsilon_instance.status;

        let instance_name = instance_metadata.name.as_ref().unwrap();

        let template_name = &instance_spec.template;

        if let Ok(pod_option) = pod_api.get_opt(instance_name).await {
            match pod_option {
                None => {
                    let template = template_provider.get_template(template_name).await.unwrap();

                    let instance_type = &template.t;
                    let instance_resource = &template.resources;

                    let mut labels = BTreeMap::new();
                    labels.insert(String::from("epsilon.fr/type"), instance_type.to_string());

                    let pod = Pod {
                        metadata: ObjectMeta {
                            name: Some(instance_name.clone()),
                            owner_references: Some(vec![instance_owner_reference]),
                            labels: Some(labels),
                            ..Default::default()
                        },
                        spec: Some(PodSpec {
                            restart_policy: Some(String::from("Never")),
                            containers: vec![Container {
                                name: String::from("main"),
                                image: Some(Self::get_image(&template_name)),
                                image_pull_policy: Some(String::from("Always")),
                                env_from: Some(vec![
                                    EnvFromSource {
                                        config_map_ref: Some(ConfigMapEnvSource {
                                            name: Some(String::from("epsilon-configuration")),
                                            optional: Some(false),
                                        }),
                                        ..Default::default()
                                    },
                                    EnvFromSource {
                                        config_map_ref: Some(ConfigMapEnvSource {
                                            name: Some(String::from(
                                                "epsilon-configuration-instance",
                                            )),
                                            optional: Some(true),
                                        }),
                                        ..Default::default()
                                    },
                                ]),
                                ports: Some(instance_type.get_associated_ports()),
                                resources: Some(instance_resource.kube_resources()),
                                readiness_probe: Some(Probe {
                                    initial_delay_seconds: Some(5),
                                    period_seconds: Some(1),
                                    success_threshold: Some(1),
                                    failure_threshold: Some(3),
                                    exec: Some(ExecAction {
                                        command: Some(
                                            vec!["cat", "epsilon_start"]
                                                .into_iter()
                                                .map(String::from)
                                                .collect(),
                                        ),
                                    }),
                                    ..Default::default()
                                }),
                                ..Default::default()
                            }],
                            ..Default::default()
                        }),
                        ..Default::default()
                    };

                    pod_api.create(&PostParams::default(), &pod).await?;
                }
                Some(pod) => {
                    let pod_metadata = &pod.metadata;

                    let pod_spec = pod.spec.as_ref().unwrap();
                    let pod_status = pod.status.as_ref().unwrap();

                    let pod_ip = pod_status.pod_ip.as_ref();
                    let pod_conditions = pod_status.conditions.as_ref().unwrap();
                    let pod_phase = pod_status.phase.as_ref().unwrap();

                    let is_starting = pod_phase == "Pending";
                    let is_running = pod_phase == "Running";
                    let is_ready = pod_conditions
                        .into_iter()
                        .any(|condition| condition.type_ == "Ready" && condition.status == "True");

                    let state = if is_starting
                        || !instance_status.is_some()
                        || (!instance_status.as_ref().unwrap().start && is_running && !is_ready)
                    {
                        EpsilonState::Starting
                    } else if is_running && is_ready {
                        EpsilonState::Running
                    } else {
                        EpsilonState::Stopping
                    };

                    let new_status = match instance_status {
                        None => {
                            let template =
                                template_provider.get_template(template_name).await.unwrap();

                            let instance_type = &template.t;
                            let instance_resource = &template.resources;

                            EpsilonInstanceStatus {
                                ip: pod_ip.cloned(),

                                template: String::from(template_name),
                                t: instance_type.clone(),

                                hub: template_provider.is_hub(&template),

                                content: String::from(""),

                                slots: template.slots,

                                close: false,
                                start: state == EpsilonState::Running,

                                state,
                            }
                        }
                        Some(status) => {
                            let mut new_status = status.clone();

                            new_status.ip = pod_ip.cloned();
                            new_status.state = state;

                            new_status
                        }
                    };

                    epsilon_instance_api
                        .patch_status(
                            instance_name,
                            &PatchParams::default(),
                            &Patch::Merge(json!({ "status": new_status })),
                        )
                        .await?;

                    let state = new_status.state;
                    let close = new_status.close;

                    if state == EpsilonState::Stopping && !close {
                        epsilon_instance_api
                            .patch_status(
                                instance_name,
                                &PatchParams::default(),
                                &Patch::Merge(json!({ "status": { "close": true } })),
                            )
                            .await?;

                        epsilon_instance_api
                            .delete(instance_name, &DeleteParams::default())
                            .await?;
                    }
                }
            }
        };

        Ok(Action::await_change())
    }

    fn on_error(error: &Error, _context: Arc<Context>) -> Action {
        warn!("Reconciliation error: {:?}", error);
        Action::await_change()
    }

    fn get_image(template: &str) -> String {
        format!(
            "{}/{}",
            env::var("HOST_REGISTRY").unwrap_or(String::from("dev.registry.epsilon.local")),
            template
        )
    }

    pub async fn create_epsilon_instance(&self, template_name: &str) -> EResult<EpsilonInstance> {
        let epsilon_instance_api = &self.context.epsilon_instance_api;
        let template_provider = &self.context.template_provider;

        let template = template_provider.get_template(template_name).await?;
        let instance_type = &template.t;

        let is_hub = template_provider.is_hub(&template);

        let epsilon_instance = EpsilonInstance {
            metadata: ObjectMeta {
                generate_name: Some(format!("{}-", template_name)),
                ..Default::default()
            },
            spec: EpsilonInstanceSpec {
                template: String::from(template_name),
            },
            status: None,
        };

        Ok(epsilon_instance_api
            .create(&PostParams::default(), &epsilon_instance)
            .await?)
    }

    pub fn get_epsilon_instance_api(&self) -> &Api<EpsilonInstance> {
        &self.context.epsilon_instance_api
    }

    pub fn get_epsilon_instance_store(&self) -> &Store<EpsilonInstance> {
        &self.store
    }
}
