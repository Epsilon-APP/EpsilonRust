use crate::k8s::label::Label;

use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::version::Info;
use std::sync::Arc;

use k8s_openapi::serde_json;
use k8s_openapi::serde_json::{json, Value};

use kube::api::{DeleteParams, ListParams, Patch, PatchParams, PostParams};
use kube::{Api, Client, Error};

pub struct Kube {
    namespace: String,
    info: Info,
    pods: Api<Pod>,
}

impl Kube {
    pub async fn new(namespace: &str) -> Arc<Kube> {
        let client = Client::try_default()
            .await
            .expect("Failed to create kube client");

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
        port: u16,
    ) -> Result<Pod, Error> {
        let default_label = self.get_default_label();

        let mut labels_map = serde_json::Map::new();

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
                    "image": format!("{}/{}", "127.0.0.1:5000", template),
                    "imagePullPolicy": "Always",
                    "ports": [
                        {
                            "containerPort": port,
                            "protocol": "TCP"
                        }
                    ],
                    "readinessProbe": {
                        "initialDelaySeconds": 5,
                        "periodSeconds": 1,
                        "successThreshold": 1,
                        "failureThreshold": 3,
                        "exec": {
                            "command": ["cat", "epsilon_start"]
                        }
                    }
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
        self.get_pods(Some(&vec![self.get_default_label()]), limit_option)
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

    fn get_default_label(&self) -> Label {
        Label::new("epsilon.fr/main", "true")
    }
}
