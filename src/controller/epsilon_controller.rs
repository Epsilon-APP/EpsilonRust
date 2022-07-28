use crate::controller::context::Context;
use crate::controller::definitions::epsilon_instance::{
    EpsilonInstance, EpsilonInstanceSpec, EpsilonInstanceStatus,
};
use crate::controller::definitions::epsilon_queue::EpsilonQueue;
use crate::epsilon::epsilon_error::EpsilonError;
use crate::epsilon::server::instances::common::state::EpsilonState;
use crate::TemplateProvider;
use futures::stream::StreamExt;
use k8s_openapi::api::core::v1::{
    ConfigMapEnvSource, Container, EnvFromSource, ExecAction, Pod, PodSpec, Probe,
};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::api::{DeleteParams, ListParams, Patch, PatchParams, PostParams};
use kube::runtime::controller::Action;
use kube::runtime::reflector::{ObjectRef, Store};
use kube::runtime::Controller;
use kube::{Api, Client, Config};
use kube::{Error, Resource};
use serde_json::json;
use std::borrow::BorrowMut;
use std::collections::BTreeMap;
use std::env;
use std::sync::Arc;

pub struct EpsilonController {
    context: Arc<Context>,
    store: Arc<Store<EpsilonInstance>>,
}

impl EpsilonController {
    pub async fn new(
        namespace: &str,
        instance_provider: &Arc<TemplateProvider>,
    ) -> Arc<EpsilonController> {
        let config = Config::infer().await.expect("Failed to load kube config");
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
                        Ok(_) => {}
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

        let instance_spec = &epsilon_instance.spec;
        let instance_status = epsilon_instance.status.clone();

        let instance_name = epsilon_instance.get_name();
        let template_name = &instance_spec.template;

        if let Ok(pod_option) = pod_api.get_opt(&instance_name).await {
            match pod_option {
                None => {
                    let template = template_provider.get_template(template_name).await.unwrap();

                    let instance_type = &template.t;
                    let instance_resource = &template.resources;

                    let mut labels = BTreeMap::new();
                    labels.insert(
                        String::from("epsilon.fr/instance"),
                        instance_type.to_string(),
                    );

                    let pod = Pod {
                        metadata: ObjectMeta {
                            name: Some(instance_name),
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
                    let pod_status = pod.status.as_ref().unwrap();

                    let pod_ip = pod_status.pod_ip.as_ref().cloned();

                    if let Some(pod_conditions) = pod_status.conditions.as_ref() {
                        let pod_phase = pod_status.phase.as_ref().unwrap();

                        let is_starting = pod_phase == "Pending";
                        let is_running = pod_phase == "Running";
                        let is_ready = pod_conditions.into_iter().any(|condition| {
                            condition.type_ == "Ready" && condition.status == "True"
                        });

                        let state = if is_starting
                            || !instance_status.is_some()
                            || (is_running && !is_ready)
                        {
                            EpsilonState::Starting
                        } else if is_running && is_ready {
                            EpsilonState::Running
                        } else {
                            EpsilonState::Stopping
                        };

                        let mut new_status = match instance_status {
                            None => {
                                let template =
                                    template_provider.get_template(template_name).await.unwrap();

                                let template_type = template.t.clone();

                                EpsilonInstanceStatus {
                                    ip: pod_ip,

                                    template: template_name.to_owned(),
                                    t: template_type,

                                    hub: template_provider.is_hub(&template),

                                    content: String::from(""),

                                    slots: template.slots,

                                    close: state == EpsilonState::Stopping,

                                    state,
                                }
                            }
                            Some(mut status) => {
                                status.ip = pod_ip;
                                status.state = state;

                                status
                            }
                        };

                        epsilon_instance_api
                            .patch_status(
                                &instance_name,
                                &PatchParams::default(),
                                &Patch::Merge(json!({ "status": new_status })),
                            )
                            .await?;

                        let state = &new_status.state;
                        let close = &new_status.close;

                        if *state == EpsilonState::Stopping && !*close {
                            new_status.close = true;

                            epsilon_instance_api
                                .patch_status(
                                    &instance_name,
                                    &PatchParams::default(),
                                    &Patch::Merge(json!({ "status": new_status })),
                                )
                                .await?;

                            epsilon_instance_api
                                .delete(&instance_name, &DeleteParams::default())
                                .await?;
                        }
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

    pub async fn create_epsilon_instance(
        &self,
        template_name: &str,
    ) -> Result<EpsilonInstance, EpsilonError> {
        let epsilon_instance_api = &self.context.epsilon_instance_api;

        let epsilon_instance = EpsilonInstance {
            metadata: ObjectMeta {
                generate_name: Some(format!("{}-", template_name)),
                ..Default::default()
            },
            spec: EpsilonInstanceSpec {
                template: template_name.to_owned(),
            },
            status: None,
        };

        let instance_result = epsilon_instance_api
            .create(&PostParams::default(), &epsilon_instance)
            .await
            .map_err(|_| EpsilonError::CreateInstanceError(template_name.to_owned()));

        Ok(instance_result?)
    }

    pub async fn in_game_epsilon_instance(&self, instance_name: &str) -> Result<(), EpsilonError> {
        let epsilon_instance_api = &self.context.epsilon_instance_api;
        let store = &self.store;

        if let Some(epsilon_instance) = store.get(&ObjectRef::new(instance_name)) {
            let mut instance_status = epsilon_instance
                .status
                .as_ref()
                .ok_or(EpsilonError::RetrieveStatusError)?
                .clone();

            instance_status.state = EpsilonState::InGame;

            epsilon_instance_api
                .patch_status(
                    instance_name,
                    &PatchParams::default(),
                    &Patch::Merge(json!({ "status": instance_status })),
                )
                .await
                .map_err(|_| EpsilonError::RemoveInstanceError(instance_name.to_owned()))?;
        }

        Ok(())
    }

    pub fn get_epsilon_instance_api(&self) -> Api<EpsilonInstance> {
        self.context.epsilon_instance_api.clone()
    }

    pub fn get_epsilon_instance_store(&self) -> &Store<EpsilonInstance> {
        &self.store
    }
}
