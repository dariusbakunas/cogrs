pub mod group;
pub mod host;
pub mod manager;
mod parser;
mod patterns;
pub mod utils;
pub mod vars;
pub mod yml;

use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct HostGroup {
    hosts: Option<HashMap<String, Value>>,
    vars: Option<HashMap<String, Value>>,
}

pub struct DataLoader {}

struct VariableManager {}

pub fn merge_yaml_values(a: &mut Value, b: Value) {
    if b.is_null() {
        return;
    }
    match (a, b) {
        (Value::Mapping(ref mut map_a), Value::Mapping(map_b)) => {
            for (key, value_b) in map_b {
                match map_a.get_mut(&key) {
                    Some(value_a) => {
                        // Recursively merge if both values are mappings
                        merge_yaml_values(value_a, value_b);
                    }
                    None => {
                        // Insert if the key doesn't exist in map_a
                        map_a.insert(key, value_b);
                    }
                }
            }
        }
        // If `a` is not a mapping or they are different types, replace `a` entirely with `b`
        (a_val, b_val) => {
            *a_val = b_val;
        }
    }
}
