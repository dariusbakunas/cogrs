use crate::inventory::group::Group;
use crate::inventory::host::Host;
use crate::inventory::yml::parse_yaml_file;
use indexmap::IndexMap;
use log::debug;
use regex::Regex;
use std::fs;
use std::path::Path;

pub struct InventoryParser;

impl InventoryParser {}

impl InventoryParser {
    pub fn parse_source(
        source: &str,
        groups: &mut IndexMap<String, Group>,
        hosts: &mut IndexMap<String, Host>,
    ) -> anyhow::Result<()> {
        debug!("Examining source {}", source);
        let path = Path::new(source);

        if !path.exists() {
            // TODO: this is not a path, could be a host list separated by commas
            return Ok(());
        }

        if path.is_dir() {
            InventoryParser::parse_directory(path, groups, hosts)?;
        } else {
            InventoryParser::parse_file(path, groups, hosts)?;
        }

        Ok(())
    }

    fn parse_directory(
        dir_path: &Path,
        groups: &mut IndexMap<String, Group>,
        hosts: &mut IndexMap<String, Host>,
    ) -> anyhow::Result<()> {
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

                    InventoryParser::parse_source(entry_str, groups, hosts)?;
                }
            }
        }

        Ok(())
    }

    fn parse_file(
        file_path: &Path,
        groups: &mut IndexMap<String, Group>,
        hosts: &mut IndexMap<String, Host>,
    ) -> anyhow::Result<()> {
        debug!("Parsing inventory file: {}", file_path.display());

        if let Some(extension) = file_path.extension() {
            match extension.to_str() {
                Some("yml" | "yaml") => parse_yaml_file(file_path, groups, hosts)?,
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
