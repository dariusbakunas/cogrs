use crate::inventory::group::Group;
use crate::inventory::host::Host;
use crate::inventory::utils::parse_host_pattern;
use anyhow::{bail, Result};
use log::{debug, error, info, warn};
use regex::Regex;
use serde_yaml;
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

fn get_value_type(val: &Value) -> &str {
    match val {
        Value::String(_) => "String",
        Value::Null => "Null",
        Value::Bool(_) => "Bool",
        Value::Number(_) => "Number",
        Value::Sequence(_) => "Sequence",
        Value::Mapping(_) => "Mapping",
        Value::Tagged(_) => "Tagged",
    }
}

pub struct InventoryManager {
    groups: HashMap<String, Group>,
    hosts: HashMap<String, Host>,
}

impl InventoryManager {
    pub fn new() -> Self {
        InventoryManager {
            groups: HashMap::new(),
            hosts: HashMap::new(),
        }
    }

    pub fn list_hosts(&self) -> Vec<String> {
        let mut hosts: Vec<String> = self.hosts.keys().cloned().collect();
        hosts.sort();
        hosts
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
        let group = self
            .groups
            .entry(group_name.to_string())
            .or_insert(Group::new(group_name));

        for (key, val) in data {
            if let Value::String(key) = key {
                match key.as_str() {
                    "vars" => {
                        info!("parsing vars");
                    }
                    "hosts" => {
                        if let Value::Mapping(val) = val {
                            for (key, val) in val {
                                if let Value::String(host_pattern) = key {
                                    let hosts = parse_host_pattern(host_pattern)?;
                                    for host_name in hosts {
                                        let host = self
                                            .hosts
                                            .entry(host_name.to_string())
                                            .or_insert(Host::new(&host_name));
                                        group.add_host(&host.name);
                                    }
                                }
                            }
                        } else {
                            error!("YAML group has invalid structure, hosts should be a dictionary, got: {}", get_value_type(&val))
                        }
                    }
                    "children" => {
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
                    }
                    &_ => {
                        warn!("Skipping unexpected key \"{key}\" in group \"{group_name}\", only \"vars\", \"children\" and \"hosts\" are valid");
                    }
                }
            }
        }

        Ok(group)
    }

    fn parse_source(&mut self, source: &str) -> Result<()> {
        debug!("Examining source {}", source);
        let path = Path::new(source);

        if !path.exists() {
            // TODO: this is not a path, could be a host list separated by commas
            return Ok(());
        }

        if path.is_dir() {
            self.parse_directory(path)?;
        } else {
            self.parse_file(path)?;
        }

        Ok(())
    }

    fn parse_directory(&mut self, dir_path: &Path) -> Result<()> {
        debug!(
            "Loading inventory files in directory: {}",
            dir_path.display()
        );
        let paths = fs::read_dir(dir_path)?;

        let exclude_pattern = Regex::new(r"^(?:\.|host_vars|group_vars|vars_plugins)(/|$)")?;

        for path in paths {
            if let Ok(entry) = path {
                let entry_path = entry.path();
                let entry_str = entry_path.to_str().unwrap_or("");

                if let Some(file_name) = entry_path.file_name() {
                    let filename = file_name.to_str().unwrap_or("");

                    if exclude_pattern.is_match(filename) {
                        debug!("Skipping excluded file or directory: {}", entry_str);
                        continue;
                    }

                    self.parse_source(entry_str)?;
                }
            }
        }

        Ok(())
    }

    fn parse_file(&mut self, file_path: &Path) -> Result<()> {
        debug!("Parsing inventory file: {}", file_path.display());

        if let Some(extension) = file_path.extension() {
            match extension.to_str() {
                Some("yml" | "yaml") => self.parse_yaml_file(file_path)?,
                _ => {
                    debug!(
                        "Skipping file due to incompatible extension: {}",
                        file_path.display()
                    );
                }
            }
        }

        Ok(())
    }

    fn parse_yaml_file(&mut self, file_path: &Path) -> Result<()> {
        let file = std::fs::File::open(file_path)?;
        let data: Value = serde_yaml::from_reader(file)?;

        match data {
            Value::Mapping(groups) => {
                for (key, val) in groups {
                    if let Value::String(group_name) = key {
                        if let Value::Mapping(group_data) = val {
                            self.parse_group(&group_name, &group_data)?;
                        } else {
                            error!(
                            "YAML group has invalid structure, it should be a dictionary, got: {}",
                            get_value_type(&val)
                        );
                        }
                    }
                }
            }
            _ => {
                error!(
                    "YAML inventory has invalid structure, it should be a dictionary, got: {}",
                    get_value_type(&data)
                );
            }
        }

        Ok(())
    }
}
