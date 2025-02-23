use anyhow::{bail, Result};
use indexmap::IndexMap;
use log::warn;
use minijinja::Environment;
use once_cell::sync::Lazy;
use serde::de::DeserializeOwned;
use serde_yaml::Value;
use std::path::PathBuf;
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
        let env = Environment::new();
        let template_context = self.load_template_context();
        let rendered_config = self.render_with_jinja(config_map, &env, &template_context)?;
        self.base_defs.extend(rendered_config);
        Ok(())
    }

    fn render_with_jinja(
        &self,
        config_map: IndexMap<String, Value>,
        env: &Environment,
        context_data: &std::collections::HashMap<String, String>,
    ) -> Result<IndexMap<String, Value>> {
        let mut rendered_map = IndexMap::new();

        for (key, value) in config_map {
            let rendered_value = self.recursively_render_value(&value, env, context_data)?;
            rendered_map.insert(key, rendered_value);
        }

        Ok(rendered_map)
    }

    fn recursively_render_value(
        &self,
        value: &Value,
        env: &Environment,
        context_data: &std::collections::HashMap<String, String>,
    ) -> Result<Value> {
        match value {
            Value::String(s) => {
                if s.contains("{{") {
                    // Create an inline template and render it
                    let template = env.template_from_str(s).map_err(|err| {
                        anyhow::anyhow!("Failed to parse Jinja template: {}", err)
                    })?;

                    let rendered = template.render(context_data).map_err(|err| {
                        anyhow::anyhow!("Failed to render Jinja template: {}", err)
                    })?;

                    Ok(Value::String(rendered))
                } else {
                    // If string doesn't contain Jinja syntax, leave it as-is
                    Ok(Value::String(s.clone()))
                }
            }
            Value::Mapping(map) => {
                // Convert the Mapping into a newly rendered Mapping
                let mut rendered_map = serde_yaml::Mapping::new();
                for (key, val) in map {
                    let rendered_key = self.recursively_render_value(key, env, context_data)?;
                    let rendered_val = self.recursively_render_value(val, env, context_data)?;
                    rendered_map.insert(rendered_key, rendered_val);
                }
                Ok(Value::Mapping(rendered_map))
            }
            Value::Sequence(seq) => {
                // Recursively process sequences
                let rendered_seq = seq
                    .iter()
                    .map(|val| self.recursively_render_value(val, env, context_data))
                    .collect::<Result<Vec<Value>>>()?;
                Ok(Value::Sequence(rendered_seq))
            }
            _ => {
                // Leave other types unmodified
                Ok(value.clone())
            }
        }
    }

    fn load_template_context(&self) -> std::collections::HashMap<String, String> {
        let mut context = std::collections::HashMap::new();

        for key in self.base_defs.keys() {
            if let Ok(Some((value, _))) = self.get_config_value::<String>(key) {
                context.insert(key.clone(), value);
            } else if let Ok(Some((value, _))) = self.get_config_value::<usize>(key) {
                // Handle numeric types (converted to strings for Jinja)
                context.insert(key.clone(), value.to_string());
            } else if let Ok(Some((value, _))) = self.get_config_value::<bool>(key) {
                context.insert(key.clone(), value.to_string());
            }
        }

        context
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
                                mapping
                                    .get("type")
                                    .filter(|v| v.is_string())
                                    .and_then(|v| v.as_str().map(|s| s.to_string())),
                            );
                        }
                    }
                }
            }
        }

        // Fall back to the "default" value
        if let Some(default_value) = mapping.get("default") {
            return parse_config_value::<T>(
                default_value.clone(),
                ConfigOrigin::Default,
                key,
                mapping
                    .get("type")
                    .filter(|v| v.is_string())
                    .and_then(|v| v.as_str().map(|s| s.to_string())),
            );
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
    value_type: Option<String>,
) -> Result<Option<(T, ConfigOrigin)>> {
    if value_type.as_deref() == Some("path") {
        if let Value::String(path_str) = &value {
            let expanded_paths: Result<Vec<String>> = path_str
                .split(':') // Split paths by colon
                .map(|path| {
                    if path.starts_with('~') {
                        // Expand '~' to the user's home directory
                        if let Some(home_dir) = dirs::home_dir() {
                            Ok(path.replacen('~', home_dir.to_str().unwrap_or_default(), 1))
                        } else {
                            bail!("Failed to expand '~/': Home directory could not be determined.");
                        }
                    } else {
                        Ok(path.to_string()) // Path does not start with '~', leave it as is
                    }
                })
                .collect();

            // Join expanded paths back into a colon-separated string
            let expanded_path_str = expanded_paths?.join(":");
            return Ok(Some((
                serde_yaml::from_value::<T>(Value::String(expanded_path_str))?, // Deserialize expanded PathBuf
                origin,
            )));
        }
    }

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
            "COGRS_HOME".to_string(),
            serde_yaml::Value::Mapping(
                serde_yaml::from_str(
                    r#"
                    env: [{name: COGRS_HOME}]
                    default: '/home/cogrs'
                "#,
                )
                .unwrap(),
            ),
        );

        base_defs.insert(
            "NUM_THREADS".to_string(),
            serde_yaml::Value::Mapping(serde_yaml::from_str("default: 4").unwrap()),
        );

        base_defs.insert(
            "ENABLE_DEBUG".to_string(),
            serde_yaml::Value::Mapping(serde_yaml::from_str("default: false").unwrap()),
        );

        base_defs.insert(
            "SERVER_CONFIG".to_string(),
            serde_yaml::Value::Mapping(
                serde_yaml::from_str("default: { HOST: '127.0.0.1', PORT: 8080 }").unwrap(),
            ),
        );

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

    #[test]
    fn test_load_template_context_with_get_config_value() {
        let mut manager = setup_test_manager();

        let context = manager.load_template_context();

        // Validate the context contains expected values
        assert_eq!(context.get("COGRS_HOME"), Some(&"/home/cogrs".to_string()));
        assert_eq!(context.get("NUM_THREADS"), Some(&"4".to_string())); // Numbers are converted to strings
        assert_eq!(context.get("ENABLE_DEBUG"), Some(&"false".to_string())); // Numbers are converted to strings
        assert!(!context.contains_key("SERVER_CONFIG")); // Complex types, skipped
    }

    #[tokio::test]
    async fn test_render_jinja_mapping_templates() {
        std::env::set_var("COGRS_HOME", "/home/cogrs");

        let mut manager = setup_test_manager();

        manager.base_defs.insert(
            "key1".to_string(),
            serde_yaml::from_str(
                r#"
                default: '{{ COGRS_HOME ~ "/plugins/become" }}'
                "#,
            )
            .unwrap(),
        );

        manager.base_defs.insert(
            "key2".to_string(),
            serde_yaml::from_str(
                r#"
                default: 42
                "#,
            )
            .unwrap(),
        );

        let env = Environment::new();
        let template_context = manager.load_template_context();
        let rendered_config = manager
            .render_with_jinja(manager.base_defs.clone(), &env, &template_context)
            .unwrap();
        manager.base_defs = rendered_config;

        let (key1, _) = manager.get_config_value::<String>("key1").unwrap().unwrap();
        let (key2, _) = manager.get_config_value::<usize>("key2").unwrap().unwrap();

        assert_eq!(key1, "/home/cogrs/plugins/become".to_owned());

        assert_eq!(key2, 42);
    }
}
