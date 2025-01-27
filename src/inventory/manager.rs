use super::group::Group;
use super::host::Host;
use super::yml::parse_yaml_file;
use crate::constants::LOCALHOST;
use anyhow::Result;
use indexmap::IndexMap;
use log::{debug, warn};
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

pub struct InventoryManager {
    groups: IndexMap<String, Group>,
    hosts: IndexMap<String, Host>,
}

impl InventoryManager {
    pub fn new() -> Self {
        InventoryManager {
            groups: IndexMap::new(),
            hosts: IndexMap::new(),
        }
    }

    pub fn list_hosts(&self) -> Vec<Host> {
        let hosts: Vec<Host> = self.hosts.values().cloned().collect();
        hosts
    }

    fn get_pattern_priority(&self, pattern: &str) -> usize {
        match pattern.chars().next() {
            Some('!') => 2, // Exclude patterns get lowest priority
            Some('&') => 1, // Intersection patterns get medium priority
            _ => 0,         // Include patterns get highest priority
        }
    }

    fn resolve_patterns(&self, patterns: &[String]) -> Vec<String> {
        let mut resolved_patterns = Vec::new();

        for pattern in patterns {
            if let Some(file_patterns) = self.read_patterns_from_file(pattern) {
                resolved_patterns.extend(file_patterns);
            } else {
                resolved_patterns.push(pattern.clone());
            }
        }

        resolved_patterns
    }

    fn read_patterns_from_file(&self, pattern: &str) -> Option<Vec<String>> {
        if !pattern.starts_with('@') {
            return None;
        }

        let filename = &pattern[1..];
        let path = Path::new(filename);

        if !path.exists() || !path.is_file() {
            warn!(
                "Pattern '{}' references a file that doesn't exist: {}",
                pattern, filename
            );
            return None;
        }

        match fs::read_to_string(path) {
            Ok(content) => {
                let lines = content
                    .lines()
                    .map(|line| line.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect();
                Some(lines)
            }
            Err(err) => {
                warn!("Could not read file '{}': {}", filename, err);
                None
            }
        }
    }

    pub fn filter_hosts(&self, limit: Option<&str>, pattern: &str) -> Result<Vec<Host>> {
        if self.hosts.is_empty() && LOCALHOST.contains(&pattern) {
            warn!("Provided hosts list is empty, only localhost is available. Note that the implicit localhost does not match 'all'");
        }

        let stripped_pattern = pattern.trim_start_matches('\'').trim_end_matches('\'');

        let mut combined_patterns = Vec::new();

        combined_patterns.extend(stripped_pattern.split(',').map(|p| p.trim().to_string()));

        if let Some(limit) = limit {
            let stripped_limit = limit.trim_start_matches('\'').trim_end_matches('\'');
            combined_patterns.extend(stripped_limit.split(',').map(|p| p.trim().to_string()));
        }

        // Resolve all patterns, expanding from files where necessary
        let mut resolved_patterns = self.resolve_patterns(&combined_patterns);

        // Sort resolved patterns by priority
        resolved_patterns.sort_by(|a, b| {
            self.get_pattern_priority(a)
                .cmp(&self.get_pattern_priority(b))
        });

        // Apply resolved patterns to filter hosts
        let mut selected_hosts = self.apply_patterns(resolved_patterns)?;

        if let Some(limit) = limit {
            let patterns: Vec<String> = limit.split(',').map(|p| p.trim().to_string()).collect();
            let resolved_patterns = self.resolve_patterns(&patterns);
            let limit_hosts = self.apply_patterns(resolved_patterns)?;
            let limit_host_set: HashSet<String> = limit_hosts.into_iter().collect();

            selected_hosts = selected_hosts
                .into_iter()
                .filter(|host| limit_host_set.contains(host))
                .collect();
        }

        // TODO: handle localhost and all

        // Map host names to Host objects, filtering out invalid entries
        Ok(selected_hosts
            .iter()
            .filter_map(|host| self.hosts.get(host).cloned())
            .collect())
    }

    fn apply_patterns(&self, patterns: Vec<String>) -> Result<Vec<String>> {
        let mut selected_hosts = Vec::new();

        for pattern in patterns {
            let matched_hosts = self.match_single_pattern(&pattern)?;

            if pattern.starts_with('!') {
                // Exclude hosts matching the pattern
                selected_hosts = selected_hosts
                    .into_iter()
                    .filter(|host| !matched_hosts.contains(host))
                    .collect();
            } else if pattern.starts_with('&') {
                // Retain only hosts that match the intersection pattern
                selected_hosts = selected_hosts
                    .into_iter()
                    .filter(|host| matched_hosts.contains(host))
                    .collect();
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

        let (expr, subscript) = self.split_subscript(stripped_pattern)?;
        let mut hosts = self.enumerate_matches(&expr)?;
        if let Some((start, end)) = subscript {
            hosts = self.apply_subscript(&hosts, start, end);
        }

        Ok(hosts)
    }

    fn apply_subscript(&self, hosts: &Vec<String>, start: i32, end: Option<i32>) -> Vec<String> {
        let len = hosts.len() as i32;

        // Handle negative indexing for `start`
        let start_idx = if start < 0 { len + start } else { start };

        // If end is not set, use just the `start`
        if end.is_none() {
            if start_idx >= 0 && start_idx < len {
                return vec![hosts[start_idx as usize].clone()];
            } else {
                return vec![];
            }
        }

        // Handle negative indexing for `end`
        let end_idx = if let Some(end_val) = end {
            if end_val < 0 {
                len + end_val
            } else {
                end_val
            }
        } else {
            len
        };

        if start_idx < 0 || start_idx >= len {
            return vec![];
        }

        if end_idx < 0 || end_idx >= len {
            return vec![];
        }

        if start_idx >= end_idx {
            return vec![];
        }

        hosts[start_idx as usize..=end_idx as usize].to_vec()
    }

    fn enumerate_matches(&self, pattern: &str) -> Result<Vec<String>> {
        let mut hosts = Vec::new();

        let matched_groups = self.match_list(self.groups.keys().cloned().collect(), pattern)?;
        for group_name in &matched_groups {
            let group = self
                .groups
                .get(group_name)
                .ok_or(anyhow::format_err!("Could not find {group_name} group"))?;
            let group_hosts = group.get_hosts(&self.groups, true)?;
            hosts.extend(group_hosts);
        }

        let special_chars = ['.', '?', '*', '['];
        if matched_groups.is_empty()
            || pattern.starts_with("~")
            || pattern.chars().any(|c| special_chars.contains(&c))
        {
            let matched_hosts = self.match_list(self.hosts.keys().cloned().collect(), pattern)?;
            hosts.extend(matched_hosts);
        }

        Ok(hosts)
    }

    fn glob_to_regex(&self, glob: &str) -> Result<String> {
        let mut regex = String::from("^");
        for ch in glob.chars() {
            match ch {
                '*' => regex.push_str(".*"),
                '?' => regex.push('.'),
                '.' | '\\' | '+' | '(' | ')' | '|' | '^' | '$' | '[' | ']' | '{' | '}' => {
                    regex.push('\\');
                    regex.push(ch);
                }
                _ => regex.push(ch),
            }
        }
        regex.push('$');
        Ok(regex)
    }

    fn match_list(&self, items: Vec<String>, pattern_str: &str) -> Result<Vec<String>> {
        // Compile patterns
        let pattern = if !pattern_str.starts_with('~') {
            Regex::new(&self.glob_to_regex(pattern_str)?)?
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

    /// Takes a pattern, checks if it has a subscript, and returns the pattern
    /// without the subscript and a (start,end) tuple representing the given
    /// subscript (or None if there is no subscript).
    fn split_subscript(&self, pattern: &str) -> Result<(String, Option<(i32, Option<i32>)>)> {
        // Do not parse regexes for enumeration info
        if pattern.starts_with('~') {
            return Ok((pattern.to_string(), None));
        }

        // Compiling the regex for pattern with subscript
        let pattern_with_subscript = Regex::new(
            r"(?x)
            ^
            (.+?)                    # A pattern expression ending with...
            \[(?:                   # A [subscript] expression comprising:
                (-?[0-9]+)|         # A single positive or negative number
                ([0-9]*)([:-])      # Or an x:y or x:- range (start can be empty; e.g., :y or :-y).
                ([0-9]*)          # End number (can be empty, can be negative).
            )]
            $
        ",
        )?;

        // Using the regex to validate and parse the input pattern
        if let Some(captures) = pattern_with_subscript.captures(pattern) {
            let trimmed_pattern = captures.get(1).map_or("", |m| m.as_str()).to_string();

            if let Some(idx_match) = captures.get(2) {
                let idx = idx_match.as_str().parse::<i32>().unwrap();
                return Ok((trimmed_pattern, Some((idx, None))));
            } else {
                let start = captures.get(3).map_or(0, |start_str| {
                    let s = start_str.as_str();
                    if s.is_empty() {
                        0
                    } else {
                        s.parse::<i32>().unwrap_or(0)
                    }
                });

                let sep = captures.get(4).map_or(":", |m| m.as_str());

                let end = captures.get(5).map_or(-1, |end_str| {
                    let s = end_str.as_str();
                    if s.is_empty() {
                        -1
                    } else {
                        s.parse::<i32>().unwrap_or(-1)
                    }
                });

                if sep == "-" {
                    println!("Warning: Use [x:y] inclusive subscripts instead of [x-y], which has been removed.");
                }

                return Ok((trimmed_pattern, Some((start, Some(end)))));
            }
        }

        Ok((pattern.to_string(), None))
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

    pub fn parse_source(&mut self, source: &str) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_split_subscript_no_subscript() {
        let inventory_manager = InventoryManager::new();

        let input_pattern = "pattern_without_subscript";
        let (parsed_pattern, subscript) = inventory_manager.split_subscript(input_pattern).unwrap();

        assert_eq!(parsed_pattern, input_pattern.to_string());
        assert!(subscript.is_none());
    }

    #[test]
    fn test_split_subscript_with_single_index_subscript() {
        let inventory_manager = InventoryManager::new();

        let input_pattern = "host[3]";
        let (parsed_pattern, subscript) = inventory_manager.split_subscript(input_pattern).unwrap();

        assert_eq!(parsed_pattern, "host".to_string());
        assert_eq!(subscript, Some((3, None)));
    }

    #[test]
    fn test_split_subscript_with_positive_range_subscript() {
        let inventory_manager = InventoryManager::new();

        let input_pattern = "host[1:4]";
        let (parsed_pattern, subscript) = inventory_manager.split_subscript(input_pattern).unwrap();

        assert_eq!(parsed_pattern, "host".to_string());
        assert_eq!(subscript, Some((1, Some(4))));
    }

    #[test]
    fn test_split_subscript_with_negative_index() {
        let inventory_manager = InventoryManager::new();

        let input_pattern = "host[-2]";
        let (parsed_pattern, subscript) = inventory_manager.split_subscript(input_pattern).unwrap();

        assert_eq!(parsed_pattern, "host".to_string());
        assert_eq!(subscript, Some((-2, None)));
    }

    #[test]
    fn test_split_subscript_with_positive_to_infinite_range() {
        let inventory_manager = InventoryManager::new();

        let input_pattern = "host[5:]";
        let (parsed_pattern, subscript) = inventory_manager.split_subscript(input_pattern).unwrap();

        assert_eq!(parsed_pattern, "host".to_string());
        assert_eq!(subscript, Some((5, Some(-1))));
    }

    #[test]
    fn test_split_subscript_with_infinite_to_negative_end_range() {
        let inventory_manager = InventoryManager::new();

        let input_pattern = "host[:-3]";
        let (parsed_pattern, subscript) = inventory_manager.split_subscript(input_pattern).unwrap();

        assert_eq!(parsed_pattern, "host".to_string());
        assert_eq!(subscript, Some((0, Some(-3))));
    }

    #[test]
    fn test_split_subscript_with_invalid_pattern() {
        let inventory_manager = InventoryManager::new();

        let input_pattern = "host[invalid]";
        let (parsed_pattern, subscript) = inventory_manager.split_subscript(input_pattern).unwrap();

        // In case of an invalid pattern, the function should return the full pattern unchanged,
        // and subscript should be None.
        assert_eq!(parsed_pattern, input_pattern.to_string());
        assert!(subscript.is_none());
    }

    #[test]
    fn test_split_subscript_with_regex_flag() {
        let inventory_manager = InventoryManager::new();

        let input_pattern = "~host_regex[3]";
        let (parsed_pattern, subscript) = inventory_manager.split_subscript(input_pattern).unwrap();

        // Regex patterns are not subjected to parsing for subscripts and remain intact.
        assert_eq!(parsed_pattern, input_pattern.to_string());
        assert!(subscript.is_none());
    }

    #[test]
    fn test_split_subscript_edge_cases() {
        let inventory_manager = InventoryManager::new();

        // Empty input
        let input_pattern_empty = "";
        let (parsed_empty, subscript_empty) = inventory_manager
            .split_subscript(input_pattern_empty)
            .unwrap();
        assert_eq!(parsed_empty, "".to_string());
        assert!(subscript_empty.is_none());

        // Special characters in pattern
        let input_pattern_special = "host[*][1]";
        let (parsed_special, subscript_special) = inventory_manager
            .split_subscript(input_pattern_special)
            .unwrap();
        assert_eq!(parsed_special, "host[*]".to_string());
        assert_eq!(subscript_special, Some((1, None)));
    }

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
