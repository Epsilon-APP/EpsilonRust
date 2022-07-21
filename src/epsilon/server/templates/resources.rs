use k8s_openapi::api::core::v1::ResourceRequirements;
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize)]
pub struct ResourcesInfo {
    pub cpu: u8,
    pub ram: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Resources {
    pub minimum: ResourcesInfo,
    pub maximum: ResourcesInfo,
}

impl Resources {
    pub fn kube_resources(&self) -> ResourceRequirements {
        ResourceRequirements {
            limits: Some(BTreeMap::from([
                (
                    String::from("cpu"),
                    Quantity(format!("{}", self.maximum.cpu)),
                ),
                (
                    String::from("memory"),
                    Quantity(format!("{}M", self.maximum.ram)),
                ),
            ])),
            requests: Some(BTreeMap::from([
                (
                    String::from("cpu"),
                    Quantity(format!("{}", self.minimum.cpu)),
                ),
                (
                    String::from("memory"),
                    Quantity(format!("{}M", self.minimum.ram)),
                ),
            ])),
        }
    }
}
