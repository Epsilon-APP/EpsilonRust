use k8s_openapi::api::core::v1::ContainerPort;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io;
use std::str::FromStr;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, JsonSchema)]
pub enum InstanceType {
    Server,
    Proxy,
}

impl InstanceType {
    pub fn get_associated_ports(&self) -> Vec<ContainerPort> {
        match self {
            InstanceType::Server => vec![ContainerPort {
                container_port: 25565,
                name: Some(String::from("server")),
                protocol: Some(String::from("TCP")),
                ..Default::default()
            }],
            InstanceType::Proxy => vec![
                ContainerPort {
                    container_port: 25577,
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
}

impl FromStr for InstanceType {
    type Err = io::Error;

    fn from_str(input: &str) -> Result<InstanceType, Self::Err> {
        match String::from(input).to_lowercase().as_str() {
            "server" => Ok(InstanceType::Server),
            "proxy" => Ok(InstanceType::Proxy),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid instance type, {}", input),
            )),
        }
    }
}

impl ToString for InstanceType {
    fn to_string(&self) -> String {
        String::from(match self {
            InstanceType::Server => "server",
            InstanceType::Proxy => "proxy",
        })
    }
}
