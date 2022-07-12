use std::io;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum InstanceType {
    Server,
    Proxy,
}

impl InstanceType {
    pub fn get_associated_ports(&self) -> Vec<(&str, u16)> {
        match self {
            InstanceType::Server => vec![("server", 25565), ("metrics", 9090)],
            InstanceType::Proxy => vec![("proxy", 25577), ("metrics", 9090)],
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
