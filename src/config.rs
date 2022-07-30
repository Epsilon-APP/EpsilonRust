use std::fs;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct EpsilonConfig {
    pub proxy: ProxyConfig,
    pub hub: HubConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ProxyConfig {
    pub template: String,
    pub minimum_proxies: u8,
}

#[derive(Serialize, Deserialize)]
pub struct HubConfig {
    pub template: String,
    pub minimum_hubs: u8,
}

impl Default for EpsilonConfig {
    fn default() -> Self {
        Self {
            proxy: ProxyConfig {
                template: String::from("waterfall"),
                minimum_proxies: 1,
            },
            hub: HubConfig {
                template: String::from("hub"),
                minimum_hubs: 1,
            },
        }
    }
}

impl EpsilonConfig {
    pub fn load(name: &str) -> Arc<EpsilonConfig> {
        Arc::new(match fs::read_to_string(name) {
            Ok(json) => {
                serde_json::from_str::<EpsilonConfig>(&json).expect("Failed to parse config")
            }
            Err(_) => {
                let config = EpsilonConfig::default();

                fs::write(
                    name,
                    serde_json::to_string_pretty(&config)
                        .expect("Failed to serialize default config"),
                )
                .expect("Failed to write default config");

                config
            }
        })
    }
}
