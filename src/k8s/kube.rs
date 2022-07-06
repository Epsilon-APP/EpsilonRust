use crate::k8s::label::Label;
use std::collections::HashMap;
use std::env;

use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::version::Info;
use std::sync::Arc;
use std::time::Duration;

use k8s_openapi::serde_json;
use k8s_openapi::serde_json::{json, Value};

use crate::epsilon::server::templates::resources::Resources;
use kube::api::{DeleteParams, ListParams, Patch, PatchParams, PostParams};
use kube::{Api, Client, Config, Error};
use serde_json::Number;

pub struct Kube {
    namespace: String,
    info: Info,
    pods: Api<Pod>,
}

impl Kube {
    pub async fn new(namespace: &str) -> Arc<Kube> {
        let mut config = Config::infer().await.expect("Failed to load kube config");
        config.timeout = Some(Duration::from_secs(3));

        let client = Client::try_from(config).expect("Failed to create kube client");

        let info = client
            .apiserver_version()
            .await
            .expect("Kube API cannot ping");

        let pods = Api::namespaced(client, namespace);

        Arc::new(Self {
            namespace: String::from(namespace),
            info,
            pods,
        })
    }

    pub async fn create_epsilon_pod(
        &self,
        template: &str,
        labels_option: Option<&Vec<Label>>,
        ports: Vec<u16>,
        resources: &Resources,
    ) -> Result<Pod, Error> {
        let default_label = Label::get_default_label();

        let mut labels_map = HashMap::new();

        let key_default = String::from(default_label.get_key());
        let value_default = String::from(default_label.get_value());

        labels_map.insert(key_default, Value::String(value_default));

        if let Some(labels) = labels_option {
            for label in labels {
                let key = String::from(label.get_key());
                let value = String::from(label.get_value());

                labels_map.insert(key, Value::String(value));
            }
        }

        let mut ports_vec = Vec::new();

        for port in ports {
            let mut map = HashMap::new();

            map.insert("containerPort", Value::Number(Number::from(port)));
            map.insert("protocol", Value::String(String::from("TCP")));

            ports_vec.push(map);
        }

        let pod = serde_json::from_value(json!({
            "metadata": {
                "generateName": format!("{}-", template),
                "labels": labels_map
            },
            "spec": {
                "terminationGracePeriodSeconds": 5,
                "restartPolicy": "Never",
                "containers": [{
                    "name": "main",
                    "image": format!("{}/{}", env::var("HOST_REGISTRY").unwrap(), template),
                    "imagePullPolicy": "Always",
                    "envFrom": [{
                        "configMapRef": {
                            "name": "epsilon-configuration"
                        },
                        "configMapRef": {
                            "name": "epsilon-configuration-instance"
                        }
                    }],
                    "ports": ports_vec,
                    "resources": {
                        "requests": {
                            "cpu": format!("{}", resources.minimum.cpu),
                            "memory": format!("{}M", resources.minimum.ram)
                        },
                        "limits": {
                            "cpu": format!("{}", resources.maximum.cpu),
                            "memory": format!("{}M", resources.maximum.ram)
                        }
                    },
                    "readinessProbe": {
                        "initialDelaySeconds": 5,
                        "periodSeconds": 1,
                        "successThreshold": 1,
                        "failureThreshold": 3,
                        "exec": {
                            "command": ["cat", "epsilon_start"]
                        }
                    },
                }],
            }
        }))
        .expect("Json Error");

        let parameters = PostParams::default();

        self.pods.create(&parameters, &pod).await
    }

    pub async fn delete_pod(&self, name: &str) -> Result<(), Error> {
        let parameters = DeleteParams::default();
        self.pods.delete(name, &parameters).await?;

        Ok(())
    }

    pub async fn get_epsilon_pods(&self, limit_option: Option<u32>) -> Result<Vec<Pod>, Error> {
        self.get_pods(Some(&vec![Label::get_default_label()]), limit_option)
            .await
    }

    pub async fn get_pods(
        &self,
        labels_option: Option<&Vec<Label>>,
        limit_option: Option<u32>,
    ) -> Result<Vec<Pod>, Error> {
        let mut parameters = ListParams::default();

        if let Some(labels) = labels_option {
            let label = Label::concat(labels);
            parameters = parameters.labels(label.as_str());
        }

        if let Some(limit) = limit_option {
            parameters = parameters.limit(limit);
        }

        let pods = self.pods.list(&parameters).await?;

        Ok(pods.items)
    }

    pub async fn get_pod(&self, pod_name: &str) -> Result<Pod, Error> {
        Ok(self.pods.get(pod_name).await?)
    }

    pub async fn patch_pod(&self, name: &str, patch: &Value) -> Result<(), Error> {
        let parameters = PatchParams::default();

        self.pods
            .patch(name, &parameters, &Patch::Strategic(&patch))
            .await?;

        Ok(())
    }

    pub fn get_namespace(&self) -> &str {
        &self.namespace
    }

    pub fn get_info(&self) -> &Info {
        &self.info
    }
}
