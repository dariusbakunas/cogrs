use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use log::{debug, error, info, warn};
use serde_yaml;
use anyhow::{bail, Result};
use regex::Regex;
use serde::de::VariantAccess;
use serde_yaml::Value;
use crate::inventory::utils::parse_host_pattern;


fn get_value_type(val: &Value) -> &str {
    match val {
        Value::String(_) => {
            "String"
        },
        Value::Null => {
            "Null"
        }
        Value::Bool(_) => {
            "Bool"
        }
        Value::Number(_) => {
            "Number"
        }
        Value::Sequence(_) => {
            "Sequence"
        }
        Value::Mapping(_) => {
            "Mapping"
        }
        Value::Tagged(_) => {
            "Tagged"
        }
    }
}

pub struct InventoryManager {
    groups: HashMap<String, Group>,
    hosts: HashMap<String, Host>,
}

impl InventoryManager {
    pub fn new() -> Self {
        InventoryManager{
            groups: HashMap::new(),
            hosts: HashMap::new()
        }
    }

    pub fn list_hosts(&self) -> Vec<Host> {
        // TODO
        Vec::new()
    }

    pub fn parse_sources(&mut self, sources: Option<&Vec<String>>) -> Result<()> {
        if let Some(sources) = sources.as_ref() {
            for source in sources.iter() {
                self.parse_source(source)?;
            }
        }

        Ok(())
    }

    fn parse_group(&mut self, group_name: &str, data: &serde_yaml::Mapping) -> Result<&Group> {
        debug!("Parsing {group_name} group");
        let group = self.groups.entry(group_name.to_string()).or_insert(Group::new(group_name));

        for (key, val) in data {
            if let Value::String(key) = key {
                if key == "vars" {
                    info!("parsing vars");
                } else if key == "children" {
                    if let Value::Mapping(val) = val {
                        for (key, val) in val {
                            if let Value::String(group_name) = key {
                                if let Value::Mapping(val) = val {
                                    info!("TODO: parse child group {group_name}")
                                } else if let Value::Null = val {
                                    // child group has no vars or hosts listed
                                    info!("TODO: parse empty child group {group_name}")
                                } else {
                                    error!("YAML group has invalid structure, it should be a dictionary, got: {}", get_value_type(&val))
                                }
                            }
                        }
                    }
                } else if key == "hosts" {
                    if let Value::Mapping(val) = val {
                        for (key, val) in val {
                            if let Value::String(host_pattern) = key {
                                let hosts = parse_host_pattern(host_pattern)?;
                                for host_name in hosts {
                                    let host = self.hosts.entry(host_name.to_string()).or_insert(Host::new(&host_name));
                                    group.add_host(&host.name);
                                }
                            }
                        }
                    } else {
                        error!("YAML group has invalid structure, hosts should be a dictionary, got: {}", get_value_type(&val))
                    }
                } else {
                    warn!("Skipping unexpected key \"{key}\" in group \"{group_name}\", only \"vars\", \"children\" and \"hosts\" are valid");
                }
            }
        }

        Ok(group)
    }

    fn parse_source(&mut self, source: &str) -> Result<()> {
        debug!("Examining source {}", source);
        let path = Path::new(source);
        if path.exists() {
            if path.is_dir() {
                debug!("Loading inventory files in directory: {}", source);
                let paths = fs::read_dir(path)?;

                // Regex to match hidden files or specific directories to exclude
                let exclude_pattern = Regex::new(r"^(?:\.|host_vars|group_vars|vars_plugins)(/|$)")?;


                for path in paths {
                    if let Ok(entry) = path {
                        let entry_path = entry.path();
                        let entry_str = entry_path.to_str().unwrap_or("");

                        if let Some(f) = entry_path.file_name() {
                            let filename = f.to_str().unwrap_or("");
                            if exclude_pattern.is_match(filename) {
                                debug!("Skipping excluded file or directory: {}", entry_str);
                                continue;
                            }

                            self.parse_source(entry_str)?;
                        }
                    }
                }
            } else {
                debug!("Parsing inventory file: {}", source);
                if let Some(extension) = path.extension() {
                    if extension == "yml" || extension == "yaml" {
                        let f = std::fs::File::open(path)?;
                        let data: Value = serde_yaml::from_reader(f)?;
                        if let Value::Mapping(groups) = data {
                            for (key, val) in groups {
                                if let Value::String(group_name) = key {
                                    if let Value::Mapping(val) = val {
                                        self.parse_group(&group_name, &val)?;
                                    } else {
                                        error!("YAML group has invalid structure, it should be a dictionary, got: {}", get_value_type(&val))
                                    }
                                }
                            }
                        } else {
                            error!("YAML inventory has invalid structure, it should be a dictionary, got: {}", get_value_type(&data))
                        }
                    } else {
                        // TODO: add support for more file types
                        debug!("Skipping file due to incompatible extension: {}", source);
                        return Ok(());
                    }
                }
            }
        } else {
            // TODO: this is not path, could be a host list separated by comma
        }

        Ok(())
    }
}