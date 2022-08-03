use k8s_openapi::api::core::v1::ContainerPort;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, JsonSchema)]
pub enum InstanceType {
    Server,
    Proxy,
}

impl InstanceType {
    pub fn get_container_ports(&self) -> Vec<ContainerPort> {
        match self {
            InstanceType::Server => vec![ContainerPort {
                container_port: self.get_entry_port(),
                name: Some(String::from("server")),
                protocol: Some(String::from("TCP")),
                ..Default::default()
            }],
            InstanceType::Proxy => vec![
                ContainerPort {
                    container_port: self.get_entry_port(),
                    name: Some(String::from("proxy")),
                    protocol: Some(String::from("TCP")),
                    ..Default::default()
                },
                ContainerPort {
                    container_port: 9090,
                    name: Some(String::from("metrics")),
                    protocol: Some(String::from("TCP")),
                    ..Default::default()
                },
            ],
        }
    }

    pub fn get_entry_port(&self) -> i32 {
        match self {
            InstanceType::Server => 25565,
            InstanceType::Proxy => 25577,
        }
    }
}

impl ToString for InstanceType {
    fn to_string(&self) -> String {
        match self {
            InstanceType::Server => "server",
            InstanceType::Proxy => "proxy",
        }
        .to_owned()
    }
}
