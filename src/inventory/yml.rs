use super::group::Group;
use super::host::Host;
use super::utils::parse_host_pattern;
use anyhow::Result;
use hashbrown::HashMap;
use log::{debug, error, info, warn};
use serde_yaml;
use serde_yaml::Value;
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

pub fn parse_yaml_file(
    file_path: &Path,
    groups: &mut HashMap<String, Group>,
    hosts: &mut HashMap<String, Host>,
) -> Result<()> {
    let file = std::fs::File::open(file_path)?;
    let data: Value = serde_yaml::from_reader(file)?;

    match data {
        Value::Mapping(group_map) => {
            for (key, val) in group_map {
                if let Value::String(group_name) = key {
                    if let Value::Mapping(group_data) = val {
                        parse_group(&group_name, &group_data, groups, hosts)?;
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

fn parse_group(
    group_name: &str,
    data: &serde_yaml::Mapping,
    groups: &mut HashMap<String, Group>,
    hosts: &mut HashMap<String, Host>,
) -> anyhow::Result<()> {
    debug!("Parsing {group_name} group");

    let mut group_stack: Vec<(String, serde_yaml::Mapping, Option<String>)> =
        vec![(group_name.to_string(), data.clone(), None)];

    while let Some((current_group_name, current_data, parent_group_name)) = group_stack.pop() {
        {
            let group = groups
                .entry(current_group_name.to_string())
                .or_insert_with(|| Group::new(&current_group_name));

            for (key, val) in &current_data {
                if let Value::String(key) = key {
                    match key.as_str() {
                        "vars" => parse_group_vars(&current_group_name),
                        "hosts" => parse_group_hosts(group, val, hosts)?,
                        "children" => {
                            parse_group_children(&current_group_name, val, &mut group_stack)?
                        }
                        _ => log_unexpected_key(key, &current_group_name),
                    }
                }
            }
        }

        if let Some(parent_group_name) = parent_group_name {
            let [Some(parent_group), Some(child_group)] =
                groups.get_many_mut([&parent_group_name, &current_group_name])
            else {
                anyhow::bail!("Parent group {parent_group_name} or child group {current_group_name} does not exist in the provided groups collection")
            };

            let mut parent_group = parent_group.clone();
            let mut child_group = child_group.clone();

            parent_group.add_child_group(&mut child_group, groups, hosts)?;

            groups.insert(parent_group.name.clone(), parent_group);
            groups.insert(child_group.name.clone(), child_group);
        }
    }

    Ok(())
}

/// Parses "vars" for the group.
fn parse_group_vars(group_name: &str) {
    info!("Parsing vars in group: {group_name}");
    // TODO: Implement parsing logic for vars if needed
}

/// Parses "hosts" for the group.
fn parse_group_hosts(
    group: &mut Group,
    val: &Value,
    hosts: &mut HashMap<String, Host>,
) -> anyhow::Result<()> {
    if let Value::Mapping(val) = val {
        for (host_key, _) in val {
            if let Value::String(host_pattern) = host_key {
                let new_hosts = parse_host_pattern(host_pattern)?;
                for host_name in new_hosts {
                    let host = hosts
                        .entry(host_name.to_string())
                        .or_insert_with(|| Host::new(&host_name));
                    group.add_host(&host.name);
                    host.add_group(&group.name)
                }
            }
        }
    } else {
        error!(
            "YAML group has invalid structure, hosts should be a dictionary, got: {}",
            get_value_type(&val)
        );
    }
    Ok(())
}

/// Parses "children" for the group and adds them to the processing stack.
fn parse_group_children(
    group_name: &str,
    val: &Value,
    group_stack: &mut Vec<(String, serde_yaml::Mapping, Option<String>)>,
) -> anyhow::Result<()> {
    if let Value::Mapping(val) = val {
        for (child_key, child_val) in val {
            if let Value::String(child_group_name) = child_key {
                let child_data = match child_val {
                    Value::Mapping(data) => data.clone(),
                    Value::Null => {
                        debug!("Queueing empty child group: {child_group_name}");
                        serde_yaml::Mapping::new()
                    }
                    _ => {
                        error!(
                            "YAML group 'children' field has invalid structure, expected dictionary or null, got: {}",
                            get_value_type(&child_val)
                        );
                        continue;
                    }
                };
                debug!("Queueing child group: {child_group_name}");
                group_stack.push((
                    child_group_name.to_string(),
                    child_data,
                    Some(group_name.to_string()),
                ));
                // TODO: add this child to parent group
            }
        }
    }
    Ok(())
}

/// Logs a warning for an unexpected key in the group.
fn log_unexpected_key(key: &str, group_name: &str) {
    warn!(
        "Skipping unexpected key \"{key}\" in group \"{group_name}\", only \"vars\", \"children\" and \"hosts\" are valid"
    );
}
