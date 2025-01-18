use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Read;
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct HostGroup {
    hosts: Option<HashMap<String, Host>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Host {}

pub fn load_inventory<R: Read>(reader: R) -> Result<HashMap<String, HostGroup>> {
    let inventory: HashMap<String, HostGroup> =
        serde_yaml::from_reader(reader)?;
    Ok(inventory)
}

pub fn filter_hosts(inventory: &HashMap<String, HostGroup>, pattern: &str) -> Vec<String> {
    let mut filtered_hosts: HashSet<String> = HashSet::new();

    if pattern == "all" {
        for (_, value) in inventory.into_iter() {
            match &value.hosts {
                Some(h) => {
                    filtered_hosts.extend(h.keys().cloned());
                }
                None => {}
            }
        }
    } else {
        let patterns: Vec<&str> = pattern
            .split([':', ',']) // Split by ':' or ','
            .collect();

        for (group_name, value) in inventory.into_iter() {
            if patterns.contains(&group_name.as_str()) {
                match &value.hosts {
                    Some(h) => {
                        filtered_hosts.extend(h.keys().cloned());
                    }
                    None => {}
                }
            } else {
                match &value.hosts {
                    Some(h) => {
                        filtered_hosts.extend(
                            h.keys()
                                .filter(|key| patterns.contains(&key.as_str()))
                                .cloned(),
                        );
                    }
                    None => {}
                }
            }
        }
    }

    let mut sorted_hosts: Vec<String> = filtered_hosts.into_iter().collect();
    sorted_hosts.sort();
    sorted_hosts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap};
    use std::io::Cursor;

    // Helper function to create a HostGroup with optional hosts
    fn create_host_group(hosts: Vec<&str>) -> HostGroup {
        let hosts_map = if hosts.is_empty() {
            None
        } else {
            Some(
                hosts
                    .into_iter()
                    .map(|host| (host.to_string(), Host {}))
                    .collect(),
            )
        };
        HostGroup { hosts: hosts_map }
    }

    #[test]
    fn test_filter_hosts_all_pattern() {
        let mut hosts = HashMap::new();
        hosts.insert(
            "group1".to_string(),
            create_host_group(vec!["host1", "host2"]),
        );
        hosts.insert(
            "group2".to_string(),
            create_host_group(vec!["host3", "host4"]),
        );

        let result = filter_hosts(&hosts, "all");
        let expected: Vec<String> = ["host1", "host2", "host3", "host4"]
            .into_iter()
            .map(String::from)
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_filter_hosts_specific_group() {
        let mut hosts = HashMap::new();
        hosts.insert(
            "group1".to_string(),
            create_host_group(vec!["host1", "host2"]),
        );
        hosts.insert(
            "group2".to_string(),
            create_host_group(vec!["host3", "host4"]),
        );

        let result = filter_hosts(&hosts, "group1");
        let expected: Vec<String> = ["host1", "host2"].into_iter().map(String::from).collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_filter_hosts_multiple_groups_pattern() {
        let mut hosts = HashMap::new();
        hosts.insert(
            "group1".to_string(),
            create_host_group(vec!["host1", "host2"]),
        );
        hosts.insert("group2".to_string(), create_host_group(vec!["host3"]));
        hosts.insert("group3".to_string(), create_host_group(vec!["host4"]));

        let result = filter_hosts(&hosts, "group1,group3");
        let expected: Vec<String> = ["host1", "host2", "host4"]
            .into_iter()
            .map(String::from)
            .collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_filter_hosts_individual_host() {
        let mut hosts = HashMap::new();
        hosts.insert(
            "group1".to_string(),
            create_host_group(vec!["host1", "host2"]),
        );
        hosts.insert(
            "group2".to_string(),
            create_host_group(vec!["host3", "host4"]),
        );

        let result = filter_hosts(&hosts, "host3");
        let expected: Vec<String> = ["host3"].into_iter().map(String::from).collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_filter_hosts_group_and_host_pattern() {
        let mut hosts = HashMap::new();
        hosts.insert(
            "group1".to_string(),
            create_host_group(vec!["host1", "host2"]),
        );
        hosts.insert("group2".to_string(), create_host_group(vec!["host3"]));
        hosts.insert(
            "group3".to_string(),
            create_host_group(vec!["host4", "host5"]),
        );

        let result = filter_hosts(&hosts, "group2,host5");
        let expected: Vec<String> = ["host3", "host5"].into_iter().map(String::from).collect();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_filter_hosts_empty_hosts() {
        let mut hosts = HashMap::new();
        hosts.insert("group1".to_string(), create_host_group(vec![]));
        hosts.insert("group2".to_string(), create_host_group(vec![]));

        let result = filter_hosts(&hosts, "all");
        let expected: Vec<String> = Vec::new();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_filter_hosts_empty_pattern() {
        let mut hosts = HashMap::new();
        hosts.insert(
            "group1".to_string(),
            create_host_group(vec!["host1", "host2"]),
        );
        hosts.insert(
            "group2".to_string(),
            create_host_group(vec!["host3", "host4"]),
        );

        let result = filter_hosts(&hosts, "");
        let expected: Vec<String> = Vec::new();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_filter_hosts_no_matching_hosts() {
        let mut hosts = HashMap::new();
        hosts.insert(
            "group1".to_string(),
            create_host_group(vec!["host1", "host2"]),
        );
        hosts.insert(
            "group2".to_string(),
            create_host_group(vec!["host3", "host4"]),
        );

        let result = filter_hosts(&hosts, "group3");
        let expected: Vec<String> = Vec::new();

        assert_eq!(result, expected);
    }

    #[test]
    fn test_load_inventory_valid_data() {
        // Prepare valid YAML inventory data
        let yaml_data = r#"
        group1:
          hosts:
            host1: {}
            host2: {}
        group2:
          hosts:
            host3: {}
        "#;

        let reader = Cursor::new(yaml_data);
        let result = load_inventory(reader);

        assert!(result.is_ok());
        let inventory = result.unwrap();
        assert!(inventory.contains_key("group1"));
        assert!(inventory.contains_key("group2"));

        let mut group1_hosts = inventory["group1"]
            .hosts
            .as_ref()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<String>>();

        group1_hosts.sort();
        assert_eq!(group1_hosts, vec!["host1", "host2"]);

        let group2_hosts = inventory["group2"]
            .hosts
            .as_ref()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<String>>();
        assert_eq!(group2_hosts, vec!["host3"]);
    }

    #[test]
    fn test_load_inventory_empty_data() {
        let empty_yaml = "";

        let reader = Cursor::new(empty_yaml);
        let result = load_inventory(reader);

        assert!(result.is_ok());
        let inventory = result.unwrap();
        assert!(inventory.is_empty());
    }

    #[test]
    fn test_load_inventory_invalid_yaml() {
        let invalid_yaml = r#"
        group1:
          hosts: { invalid_yaml
        "#;


        let reader = Cursor::new(invalid_yaml);
        let result = load_inventory(reader);

        assert!(result.is_err());
    }

    #[test]
    fn test_load_inventory_non_yaml_data() {
        let non_yaml_data = "This is not YAML!";

        let reader = Cursor::new(non_yaml_data);
        let result = load_inventory(reader);

        assert!(result.is_err());
    }
}
