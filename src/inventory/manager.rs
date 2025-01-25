use super::group::Group;
use super::host::Host;
use super::yml::parse_yaml_file;
use anyhow::Result;
use hashbrown::HashMap;
use log::{debug, info, warn};
use regex::Regex;
use std::fs;
use std::path::Path;

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

    pub fn list_hosts(&self) -> Vec<Host> {
        let mut hosts: Vec<Host> = self.hosts.values().cloned().collect();
        hosts
    }

    pub fn list_groups(&self) -> Vec<Group> {
        let groups: Vec<Group> = self.groups.values().cloned().collect();
        groups
    }

    pub fn parse_sources(&mut self, sources: Option<&Vec<String>>) -> Result<()> {
        if let Some(sources) = sources.as_ref() {
            for source in sources.iter() {
                self.parse_source(source)?;
            }
        }

        Ok(())
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
                Some("yml" | "yaml") => {
                    parse_yaml_file(file_path, &mut self.groups, &mut self.hosts)?
                }
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
}
