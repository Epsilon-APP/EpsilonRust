use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct EpsilonConfig {
    proxy: ProxyConfig,
    hub: HubConfig,
}

#[derive(Serialize, Deserialize)]
pub struct ProxyConfig {
    template: String,
    minimum_proxies: u8,
}

#[derive(Serialize, Deserialize)]
pub struct HubConfig {
    template: String,
    minimum_hubs: u8,
}

impl Default for EpsilonConfig {
    fn default() -> Self {
        Self {
            proxy: ProxyConfig {
                template: String::from("proxy"),
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
    pub fn load(name: &str) -> EpsilonConfig {
        match fs::read_to_string(name) {
            Ok(json) => serde_json::from_str::<EpsilonConfig>(&json).unwrap(),
            Err(_) => {
                let config = EpsilonConfig::default();

                fs::write(name, serde_json::to_string_pretty(&config).unwrap()).unwrap();

                config
            }
        }
    }
}
