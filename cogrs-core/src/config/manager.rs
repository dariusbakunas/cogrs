use anyhow::{bail, Result};
use indexmap::IndexMap;
use log::warn;
use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use serde_yaml::Value;
use tokio::sync::Mutex;

pub struct ConfigManager {
    base_defs: IndexMap<String, Value>,
}

#[derive(Debug, PartialEq)]
pub enum ConfigOrigin {
    Env,
    Default,
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

    pub fn get_config_value<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<(T, ConfigOrigin)>> {
        // Get the value from the base definitions
        let value = match self.base_defs.get(key) {
            Some(value) => value,
            None => {
                warn!("Config key {} not found", key);
                return Ok(None);
            }
        };

        // Ensure the value is a mapping
        let mapping = match value {
            Value::Mapping(map) => map,
            _ => {
                warn!("Config key {} is not a mapping", key);
                return Ok(None);
            }
        };

        // Check for "env" overrides
        if let Some(Value::Sequence(env_list)) = mapping.get("env") {
            for item in env_list {
                if let Value::Mapping(item_map) = item {
                    if let Some(Value::String(env_key)) = item_map.get("name") {
                        if let Ok(env_value) = std::env::var(env_key) {
                            return parse_config_value::<T>(
                                Value::String(env_value),
                                ConfigOrigin::Env,
                                key,
                            );
                        }
                    }
                }
            }
        }

        // Fall back to the "default" value
        if let Some(default_value) = mapping.get("default") {
            return parse_config_value::<T>(default_value.clone(), ConfigOrigin::Default, key);
        }

        // If no valid value is found
        warn!("Config key {} has no valid value", key);
        Ok(None)
    }
}

// Utility function for deserialization and error handling
fn parse_config_value<T: DeserializeOwned>(
    value: Value,
    origin: ConfigOrigin,
    key: &str,
) -> Result<Option<(T, ConfigOrigin)>> {
    match serde_yaml::from_value::<T>(value) {
        Ok(deserialized_value) => Ok(Some((deserialized_value, origin))),
        Err(err) => bail!(
            "Failed to cast config key '{}' value to the required type: {}",
            key,
            err
        ),
    }
}

static CONFIG_LOADER: Lazy<Mutex<ConfigManager>> = Lazy::new(|| Mutex::new(ConfigManager::new()));

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    fn setup_test_manager() -> ConfigManager {
        let mut manager = ConfigManager::new();

        // Populate base_defs with sample configuration
        let mut base_defs = IndexMap::new();
        base_defs.insert(
            "key_with_env".to_string(),
            serde_yaml::from_str(
                r#"
                env:
                  - name: ENV_VAR_TEST
                default: "default_value"
                "#,
            )
            .unwrap(),
        );
        base_defs.insert(
            "key_with_missing_env".to_string(),
            serde_yaml::from_str(
                r#"
                env:
                  - name: ENV_VAR_TEST_2
                default: "default_value"
                "#,
            )
            .unwrap(),
        );
        base_defs.insert(
            "key_without_env".to_string(),
            serde_yaml::from_str(
                r#"
                default: 42
                "#,
            )
            .unwrap(),
        );
        base_defs.insert(
            "invalid_key".to_string(),
            serde_yaml::from_str(
                r#"
                invalid_field: "invalid"
                "#,
            )
            .unwrap(),
        );
        manager.base_defs = base_defs;
        manager
    }

    #[tokio::test]
    async fn test_get_config_value_env_var_present() {
        // Set the environment variable for the test
        std::env::set_var("ENV_VAR_TEST", "env_value");

        let manager = setup_test_manager();
        let result: Option<(String, ConfigOrigin)> =
            manager.get_config_value("key_with_env").unwrap();

        assert!(result.is_some());
        let (value, origin) = result.unwrap();
        assert_eq!(value, "env_value");
        assert_eq!(origin, ConfigOrigin::Env);

        // Clean up the environment variable
        std::env::remove_var("ENV_VAR_TEST");
    }

    #[tokio::test]
    async fn test_get_config_value_default_value() {
        let manager = setup_test_manager();
        let result: Option<(String, ConfigOrigin)> =
            manager.get_config_value("key_with_missing_env").unwrap();

        assert!(result.is_some());
        let (value, origin) = result.unwrap();
        assert_eq!(value, "default_value");
        assert_eq!(origin, ConfigOrigin::Default);
    }

    #[tokio::test]
    async fn test_get_config_value_no_env_no_default() {
        let manager = setup_test_manager();
        let result: Option<(i32, ConfigOrigin)> = manager.get_config_value("invalid_key").unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_config_value_wrong_type() {
        let manager = setup_test_manager();
        let result: Result<Option<(String, ConfigOrigin)>> =
            manager.get_config_value("key_without_env");

        assert!(result.is_err());
        if let Err(err) = result {
            assert!(err.to_string().contains("Failed to cast"));
        }
    }

    #[tokio::test]
    async fn test_get_config_value_key_not_found() {
        let manager = setup_test_manager();
        let result: Option<(String, ConfigOrigin)> =
            manager.get_config_value("unknown_key").unwrap();

        assert!(result.is_none());
    }
}
