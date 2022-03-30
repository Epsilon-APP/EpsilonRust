use std::io;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum InstanceType {
    Server,
    Proxy,
}

impl InstanceType {
    pub fn get_associated_port(&self) -> u16 {
        match self {
            InstanceType::Server => 25565,
            InstanceType::Proxy => 25577,
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
