use serde_json::Value;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Template {
    pub name: String,

    #[serde(rename = "type")]
    pub t: String,

    pub slots: i32,

    pub schema: String,
    pub labels: HashMap<String, Value>,
}
