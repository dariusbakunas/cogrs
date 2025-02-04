use super::group::Group;
use super::host::Host;
use crate::constants::LOCALHOST;
use crate::inventory::patterns::PatternResolver;
use crate::inventory::utils::{glob_to_regex, split_subscript};
use crate::parsing::parser::InventoryParser;
use anyhow::Result;
use indexmap::IndexMap;
use log::warn;
use regex::Regex;
use std::collections::HashSet;

pub struct HostManager;

pub struct InventoryManager {
    groups: IndexMap<String, Group>,
    hosts: IndexMap<String, Host>,
    localhost: Host,
}

impl Default for InventoryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InventoryManager {
    pub fn new() -> Self {
        let localhost = Host::new("localhost");

        InventoryManager {
            groups: IndexMap::new(),
            hosts: IndexMap::new(),
            localhost,
        }
    }

    pub fn get_host(&self, name: &str) -> Option<&Host> {
        let host = self.hosts.get(name);

        if host.is_none() && LOCALHOST.contains(&name) {
            return Some(&self.localhost);
        }

        host
    }

    pub fn parse_sources(&mut self, sources: Option<&[String]>) -> Result<()> {
        if let Some(sources) = sources.as_ref() {
            for source in sources.iter() {
                InventoryParser::parse_source(source, &mut self.groups, &mut self.hosts)?;
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn get_combined_patterns(&self, limit: Option<&str>, pattern: &str) -> Vec<String> {
        let stripped_pattern = pattern.trim_start_matches('\'').trim_end_matches('\'');
        let mut combined_patterns: Vec<String> = stripped_pattern
            .split(',')
            .map(|p| p.trim().to_string())
            .collect();

        if let Some(limit) = limit {
            let stripped_limit = limit.trim_start_matches('\'').trim_end_matches('\'');
            combined_patterns.extend(stripped_limit.split(',').map(|p| p.trim().to_string()));
        }

        combined_patterns
    }

    fn filter_with_limit(&self, selected_hosts: &[String], limit: &str) -> Result<Vec<String>> {
        let patterns: Vec<String> = limit
            .trim_start_matches('\'')
            .trim_end_matches('\'')
            .split(',')
            .map(|p| p.trim().to_string())
            .collect();
        let resolved_limit_patterns = PatternResolver::resolve_and_sort_patterns(&patterns);

        let limit_hosts = self.apply_patterns(&resolved_limit_patterns)?;
        let limit_host_set: HashSet<String> = limit_hosts.into_iter().collect();

        Ok(selected_hosts
            .iter()
            .filter(|host| limit_host_set.contains(*host))
            .cloned()
            .collect())
    }

    pub fn filter_hosts(&self, pattern: &str, limit: Option<&str>) -> Result<Vec<Host>> {
        if self.hosts.is_empty() && LOCALHOST.contains(&pattern) {
            warn!("Provided hosts list is empty, only localhost is available. Note that the implicit localhost does not match 'all'");
        }

        let mut split_pattern: Vec<String> = pattern
            .trim_start_matches('\'')
            .trim_end_matches('\'')
            .split(',')
            .map(|p| p.trim().to_string())
            .collect();

        // TODO: use combined_patterns to generate hash key for storing results in cache
        //let combined_patterns = self.get_combined_patterns(limit, pattern);

        split_pattern.sort_by(|a, b| {
            PatternResolver::get_pattern_priority(a).cmp(&PatternResolver::get_pattern_priority(b))
        });

        let mut selected_hosts = self.apply_patterns(&split_pattern)?;

        if let Some(limit) = limit {
            // only keep hosts that match limit specification
            selected_hosts = self.filter_with_limit(&selected_hosts, limit)?;
        }

        // TODO: handle localhost and all

        // Map host names to Host objects, filtering out invalid entries
        Ok(selected_hosts
            .iter()
            .filter_map(|host| self.get_host(host).cloned())
            .collect())
    }

    fn apply_patterns(&self, patterns: &[String]) -> Result<Vec<String>> {
        let mut selected_hosts = Vec::new();

        for pattern in patterns {
            let matched_hosts = self.match_single_pattern(pattern)?;

            if pattern.starts_with('!') {
                // Exclude hosts matching the pattern
                selected_hosts.retain(|host| !matched_hosts.contains(host));
            } else if pattern.starts_with('&') {
                // Retain only hosts that match the intersection pattern
                selected_hosts.retain(|host| matched_hosts.contains(host));
            } else {
                // Add hosts that match the pattern, avoiding duplicates
                for host in matched_hosts {
                    if !selected_hosts.contains(&host) {
                        selected_hosts.push(host);
                    }
                }
            }
        }

        Ok(selected_hosts)
    }

    fn match_single_pattern(&self, pattern: &str) -> Result<Vec<String>> {
        let stripped_pattern = if pattern.starts_with('!') || pattern.starts_with('&') {
            &pattern[1..]
        } else {
            pattern
        };

        let split_pattern = split_subscript(stripped_pattern)?;
        let mut hosts = self.enumerate_matches(&split_pattern.pattern)?;
        if let Some((start, end)) = split_pattern.subscript {
            hosts = self.apply_subscript(&hosts, start, end);
        }

        Ok(hosts)
    }

    fn apply_subscript(&self, hosts: &[String], start: i32, end: Option<i32>) -> Vec<String> {
        if hosts.is_empty() {
            return vec![];
        }

        let len = hosts.len() as i32;

        // Compute start index, handling negative indexing
        let start_idx = if start < 0 { len + start } else { start };

        // Compute end index, handling negative indexing, defaulting to `start` if `end` is None
        let end_idx = end
            .map(|e| if e < 0 { len + e } else { e })
            .unwrap_or(start_idx);

        // Validate indices
        if start_idx < 0 || start_idx >= len || end_idx < 0 || end_idx >= len || start_idx > end_idx
        {
            return vec![];
        }

        // Extract and return the slice of hosts
        hosts[start_idx as usize..=end_idx as usize].to_vec()
    }

    fn enumerate_matches(&self, pattern: &str) -> Result<Vec<String>> {
        let mut matches = Vec::new();

        let groups: Vec<String> = self.groups.keys().cloned().collect();
        let matched_groups = self.match_list(&groups, pattern)?;
        for group_name in &matched_groups {
            let group = self
                .groups
                .get(group_name)
                .ok_or(anyhow::format_err!("Could not find {group_name} group"))?;
            let group_hosts = group.get_hosts(&self.groups, true)?;
            matches.extend(group_hosts);
        }

        let special_chars = ['.', '?', '*', '['];
        if matched_groups.is_empty()
            || pattern.starts_with("~")
            || pattern.chars().any(|c| special_chars.contains(&c))
        {
            let hosts: Vec<String> = self.hosts.keys().cloned().collect();
            let matched_hosts = self.match_list(&hosts, pattern)?;
            matches.extend(matched_hosts);
        }

        if matches.is_empty() && LOCALHOST.contains(&pattern) {
            matches.push(pattern.to_string());
        }

        Ok(matches)
    }

    fn match_list(&self, items: &[String], pattern_str: &str) -> Result<Vec<String>> {
        if pattern_str == "all" {
            return Ok(items.to_vec());
        }

        // Compile patterns
        let pattern = if !pattern_str.starts_with('~') {
            Regex::new(&glob_to_regex(pattern_str)?)?
        } else {
            Regex::new(&pattern_str[1..])?
        };

        // Apply patterns
        let results: Vec<String> = items
            .iter()
            .filter(|item| pattern.is_match(item))
            .cloned()
            .collect();

        Ok(results)
    }

    pub fn list_groups(&self) -> Vec<Group> {
        let groups: Vec<Group> = self.groups.values().cloned().collect();
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_subscript_single_positive_index() {
        let inventory_manager = InventoryManager::new();
        let hosts = vec![
            "host1".to_string(),
            "host2".to_string(),
            "host3".to_string(),
        ];

        let result = inventory_manager.apply_subscript(&hosts, 1, None);
        assert_eq!(result, vec!["host2".to_string()]);
    }

    #[test]
    fn test_apply_subscript_single_negative_index() {
        let inventory_manager = InventoryManager::new();
        let hosts = vec![
            "host1".to_string(),
            "host2".to_string(),
            "host3".to_string(),
        ];

        let result = inventory_manager.apply_subscript(&hosts, -1, None);
        assert_eq!(result, vec!["host3".to_string()]);
    }

    #[test]
    fn test_apply_subscript_range_positive_indices() {
        let inventory_manager = InventoryManager::new();
        let hosts = vec![
            "host1".to_string(),
            "host2".to_string(),
            "host3".to_string(),
            "host4".to_string(),
        ];

        let result = inventory_manager.apply_subscript(&hosts, 1, Some(2));
        assert_eq!(result, vec!["host2".to_string(), "host3".to_string()]);
    }

    #[test]
    fn test_apply_subscript_range_negative_indices() {
        let inventory_manager = InventoryManager::new();
        let hosts = vec![
            "host1".to_string(),
            "host2".to_string(),
            "host3".to_string(),
            "host4".to_string(),
        ];

        let result = inventory_manager.apply_subscript(&hosts, -3, Some(-2));
        assert_eq!(result, vec!["host2".to_string(), "host3".to_string()]);
    }

    #[test]
    fn test_apply_subscript_single_positive_index_out_of_bounds() {
        let inventory_manager = InventoryManager::new();
        let hosts = vec!["host1".to_string(), "host2".to_string()];

        let result = inventory_manager.apply_subscript(&hosts, 5, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_subscript_negative_index_out_of_bounds() {
        let inventory_manager = InventoryManager::new();
        let hosts = vec!["host1".to_string(), "host2".to_string()];

        let result = inventory_manager.apply_subscript(&hosts, -5, None);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_subscript_range_start_greater_than_end() {
        let inventory_manager = InventoryManager::new();
        let hosts = vec![
            "host1".to_string(),
            "host2".to_string(),
            "host3".to_string(),
            "host4".to_string(),
        ];

        let result = inventory_manager.apply_subscript(&hosts, 2, Some(1));
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_subscript_empty_hosts() {
        let inventory_manager = InventoryManager::new();
        let hosts: Vec<String> = vec![];

        let result = inventory_manager.apply_subscript(&hosts, 0, Some(1));
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_subscript_full_range() {
        let inventory_manager = InventoryManager::new();
        let hosts = vec![
            "host1".to_string(),
            "host2".to_string(),
            "host3".to_string(),
            "host4".to_string(),
        ];

        let result = inventory_manager.apply_subscript(&hosts, 0, Some(3));
        assert_eq!(
            result,
            vec![
                "host1".to_string(),
                "host2".to_string(),
                "host3".to_string(),
                "host4".to_string()
            ]
        );
    }

    #[test]
    fn test_apply_subscript_with_infinite_end() {
        let inventory_manager = InventoryManager::new();
        let hosts = vec![
            "host1".to_string(),
            "host2".to_string(),
            "host3".to_string(),
            "host4".to_string(),
        ];

        let result = inventory_manager.apply_subscript(&hosts, 2, None);
        assert_eq!(result, vec!["host3".to_string()]);
    }

    #[test]
    fn test_apply_subscript_negative_to_infinite_range() {
        let inventory_manager = InventoryManager::new();
        let hosts = vec![
            "host1".to_string(),
            "host2".to_string(),
            "host3".to_string(),
            "host4".to_string(),
        ];

        let result = inventory_manager.apply_subscript(&hosts, -2, None);
        assert_eq!(result, vec!["host3".to_string()]);
    }
}
