use serde_json::Value;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::epsilon::server::instance_type::InstanceType;
use crate::epsilon::server::resources::Resources;

#[derive(Serialize, Deserialize)]
pub struct Template {
    pub name: String,
    pub parent: String,

    #[serde(rename = "type")]
    pub t: InstanceType,

    pub slots: i32,
    pub resources: Resources,

    pub labels: HashMap<String, Value>,
}
