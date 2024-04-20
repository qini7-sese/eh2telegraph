use std::{collections::HashMap, env};

use once_cell::sync::OnceCell;

static CFG_PATH: OnceCell<String> = OnceCell::new();

lazy_static::lazy_static! {
    static ref CONFIG_MAPPING: HashMap<String, serde_yaml::Value> = {
        let file_path = CFG_PATH.get_or_init(get_config_path);
        let file_content = std::fs::read_to_string(file_path).expect("config file not found");
        serde_yaml::from_str(&file_content).expect("unable to parse config file")
    };
}

fn get_config_path() -> String {
    // read from env
    if let Ok(p) = env::var("CONFIG_FILE") {
        if !p.is_empty() {
            return p;
        }
    }

    // default
    "config.yaml".to_string()
}

/// Initialize config, will panic on failure.
pub fn init(config_path: Option<String>) {
    if let Some(p) = config_path {
        let _ = CFG_PATH.set(p);
    }
    lazy_static::initialize(&CONFIG_MAPPING);
}

/// Parse struct from global config.
pub fn parse<T>(key: &str) -> serde_yaml::Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    CONFIG_MAPPING
        .get(key)
        .cloned()
        .map(|v| serde_yaml::from_value(v))
        .transpose()
}
