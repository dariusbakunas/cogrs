use crate::inventory::host::Host;
use crate::inventory::utils::difference_update_vec;
use crate::inventory::vars::Variable;
use anyhow::{bail, Result};
use indexmap::IndexMap;
use log::{debug, warn};
use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct Group {
    pub name: String,
    depth: u32,
    priority: i64,
    vars: IndexMap<String, Variable>,
    hosts: Vec<String>,
    pub child_groups: Vec<String>,
    parent_groups: Vec<String>,
}

impl Group {
    pub fn new(name: &str) -> Self {
        Group {
            name: name.to_string(),
            depth: 0,
            priority: 1,
            vars: IndexMap::new(),
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

    pub fn get_hosts(&self) -> Vec<String> {
        self.hosts.clone()
    }

    pub fn set_priority(&mut self, priority: i64) {
        self.priority = priority;
    }

    pub fn walk_relationships(
        &self,
        groups: &IndexMap<String, Group>,
        parent: bool,
    ) -> Vec<String> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut unprocessed: HashSet<String> = if parent {
            HashSet::from_iter(self.parent_groups.iter().cloned())
        } else {
            HashSet::from_iter(self.child_groups.iter().cloned())
        };
        let mut relations: Vec<String> = if parent {
            self.parent_groups.clone()
        } else {
            self.child_groups.clone()
        };

        while !unprocessed.is_empty() {
            seen.extend(unprocessed.iter().cloned());

            let mut new_unprocessed: HashSet<String> = HashSet::new();

            for group_name in &unprocessed {
                if let Some(group) = groups.get(group_name) {
                    for parent in &group.parent_groups {
                        // Only add to new_unprocessed if it hasn't already been seen:
                        if !seen.contains(parent) {
                            new_unprocessed.insert(parent.clone());
                            relations.push(parent.clone());
                        }
                    }
                } else {
                    warn!("Ancestor group {group_name} was not found in group collection");
                }
            }

            // Update unprocessed for the next iteration.
            unprocessed = new_unprocessed;
        }

        relations
    }

    pub fn get_ancestors(&self, groups: &IndexMap<String, Group>) -> Vec<String> {
        self.walk_relationships(groups, true)
    }

    pub fn get_descendants(&self, groups: &IndexMap<String, Group>) -> Vec<String> {
        self.walk_relationships(groups, false)
    }

    pub fn add_child_group(
        &mut self,
        child_group: &mut Group,
        groups: &mut IndexMap<String, Group>,
        hosts: &mut IndexMap<String, Host>,
    ) -> Result<()> {
        let child_group_name = &child_group.name;

        if self.name.eq(child_group_name) {
            bail!("Can't add group to itself: {child_group_name}!")
        }

        if self.child_groups.contains(&child_group_name.to_string()) {
            warn!(
                "Group '{child_group_name}' already exists in '{}'",
                self.name
            );
            return Ok(());
        }

        debug!("Adding child group '{child_group_name}' to '{}'", self.name);

        let start_ancestors = child_group.get_ancestors(groups);
        let mut new_ancestors = self.get_ancestors(groups);

        if new_ancestors.contains(&child_group_name.to_string()) {
            bail!("Adding group '{child_group_name}' as child to '{}' creates recursive dependency loop.", self.name);
        }

        new_ancestors.push(self.name.to_string());
        difference_update_vec(&mut new_ancestors, &start_ancestors);

        self.child_groups.push(child_group.name.clone());

        if self.depth + 1 > child_group.depth {
            child_group.depth = self.depth + 1;
            child_group.check_children_depth(groups)?;
        }

        if !child_group.parent_groups.contains(&self.name.to_string()) {
            child_group.parent_groups.push(self.name.to_string());

            for host in child_group.hosts.iter() {
                if let Some(host) = hosts.get_mut(host) {
                    host.populate_ancestors(new_ancestors.clone());
                } else {
                    bail!("Unknown host: '{host}'");
                }
            }
        }

        Ok(())
    }

    /// Sets a variable in the group's `vars` map.
    ///
    /// # Parameters
    /// - `key`: The key of the variable to be set.
    /// - `value`: The value of the variable of type `Variable`.
    ///
    /// # Behavior
    /// - If the `key` is `"ansible_group_priority"`, this method will set the group's priority
    /// - Otherwise the `value` is directly inserted or updated in `self.vars`.
    ///
    /// # Example
    /// ```rust
    /// use cogrs::inventory::group::Group;
    /// use cogrs::inventory::vars::Variable;
    /// let mut group = Group::new("example_group");
    /// let variable = Variable::String(String::from("example"));  // Replace with actual `Variable` type instance
    /// group.set_variable("key_name", variable);
    /// ```
    pub fn set_variable(&mut self, key: &str, value: Variable) {
        if key.eq("ansible_group_priority") {
            if let Variable::Number(val) = value {
                if let Some(val) = val.as_i64() {
                    self.set_priority(val);
                } else {
                    warn!("Invalid ansible_group_priority value: {:?}", val);
                }
            }
        } else {
            self.vars.insert(key.to_string(), value);
        }
    }

    fn check_children_depth(&self, groups: &mut IndexMap<String, Group>) -> Result<()> {
        let mut seen: HashSet<String> = HashSet::new();
        let mut unprocessed: HashSet<String> =
            HashSet::from_iter(self.parent_groups.iter().cloned());
        let mut depth = self.depth;
        let start_depth = self.depth;

        while !unprocessed.is_empty() {
            seen.extend(unprocessed.iter().cloned());
            depth += 1;

            let mut new_unprocessed: HashSet<String> = HashSet::new();

            for group_name in &unprocessed {
                if let Some(group) = groups.get_mut(group_name) {
                    if group.depth < depth {
                        group.depth = depth;
                        new_unprocessed.insert(group_name.to_string());
                    }
                }
            }

            unprocessed = new_unprocessed;

            if depth - start_depth > seen.len() as u32 {
                bail!(
                    "The group named '{}' has a recursive dependency loop.",
                    self.name
                );
            }
        }

        Ok(())
    }
}
