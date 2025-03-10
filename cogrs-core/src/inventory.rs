pub mod group;
pub mod host;
pub mod manager;
mod patterns;
pub mod utils;

use serde_yaml::Value;

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
