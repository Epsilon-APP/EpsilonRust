use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Resources {
    pub minimum: ResourcesInfo,
    pub maximum: ResourcesInfo,
}

#[derive(Serialize, Deserialize)]
pub struct ResourcesInfo {
    pub cpu: u8,
    pub ram: u32,
}
