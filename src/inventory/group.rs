use anyhow::{bail, Result};
use hashbrown::HashMap;
use log::{debug, warn};
use serde_yaml::Value;
use std::collections::HashSet;

#[derive(Clone)]
pub struct Group {
    pub name: String,
    depth: u32,
    vars: HashMap<String, Value>,
    hosts: Vec<String>,
    pub child_groups: Vec<String>,
    parent_groups: Vec<String>,
}

impl Group {
    pub fn new(name: &str) -> Self {
        Group {
            name: name.to_string(),
            depth: 0,
            vars: HashMap::new(),
            hosts: Vec::new(),
            child_groups: Vec::new(),
            parent_groups: Vec::new(),
        }
    }

    pub fn add_host(&mut self, host_name: &str) {
        let name = host_name.to_string();
        if !self.hosts.contains(&name) {
            self.hosts.push(name);
        }
    }

    pub fn get_ancestors(&self, groups: &HashMap<String, Group>) -> Vec<String> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut unprocessed: HashSet<String> =
            HashSet::from_iter(self.parent_groups.iter().cloned());
        let mut ancestors: Vec<String> = self.parent_groups.clone();

        while !unprocessed.is_empty() {
            seen.extend(unprocessed.iter().cloned());

            let mut new_unprocessed: HashSet<String> = HashSet::new();

            for group_name in &unprocessed {
                if let Some(group) = groups.get(group_name) {
                    for parent in &group.parent_groups {
                        // Only add to new_unprocessed if it hasn't already been seen:
                        if !seen.contains(parent) {
                            new_unprocessed.insert(parent.clone());
                            ancestors.push(parent.clone());
                        }
                    }
                } else {
                    warn!("Ancestor group {group_name} was not found in group collection");
                }
            }

            // Update unprocessed for the next iteration.
            unprocessed = new_unprocessed;
        }

        ancestors
    }

    pub fn add_parent_group(&mut self, parent_group_name: &str) {
        if !self.parent_groups.contains(&parent_group_name.to_string()) {
            self.parent_groups.push(parent_group_name.to_string());
        }
    }

    pub fn add_child_group(
        &mut self,
        child_group: &mut Group,
        groups: &mut HashMap<String, Group>,
    ) -> Result<()> {
        let child_group_name = &child_group.name;

        if self.name.eq(child_group_name) {
            bail!("Can't add group to itself: {child_group_name}!")
        }

        debug!("Adding child group '{child_group_name}' to '{}'", self.name);

        let start_ancestors = child_group.get_ancestors(groups);
        let mut new_ancestors = self.get_ancestors(groups);

        if new_ancestors.contains(&child_group_name.to_string()) {
            bail!("Adding group '{child_group_name}' as child to '{}' creates recursive dependency loop.", self.name);
        }

        new_ancestors.push(self.name.to_string());
        //difference_update(&mut new_ancestors, &start_ancestors);
        //child_group.

        self.child_groups.push(child_group.name.clone());
        child_group.add_parent_group(&self.name);

        Ok(())
    }
}
