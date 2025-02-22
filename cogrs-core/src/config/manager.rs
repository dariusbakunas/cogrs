use anyhow::Result;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use serde_yaml::Value;
use tokio::sync::Mutex;

pub struct ConfigManager {
    base_defs: IndexMap<String, Value>,
}

impl ConfigManager {
    fn new() -> Self {
        ConfigManager {
            base_defs: IndexMap::new(),
        }
    }

    pub fn instance() -> &'static Mutex<ConfigManager> {
        &CONFIG_LOADER
    }

    pub fn init(&mut self) -> Result<()> {
        let config_map = self.read_config_yaml_file()?;
        self.base_defs.extend(config_map);
        Ok(())
    }

    fn read_config_yaml_file(&self) -> Result<IndexMap<String, Value>> {
        let yaml_content = include_str!("base.yaml");
        let value: Value = serde_yaml::from_str(&yaml_content)?;

        let config_map = value
            .as_mapping()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("YAML root is not a mapping"))?
            .into_iter()
            .map(|(key, value)| {
                let key_str = key
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("YAML key is not a string"))?
                    .to_string();
                Ok((key_str, value))
            })
            .collect::<Result<IndexMap<String, Value>>>()?;

        Ok(config_map)
    }
}

static CONFIG_LOADER: Lazy<Mutex<ConfigManager>> = Lazy::new(|| Mutex::new(ConfigManager::new()));
