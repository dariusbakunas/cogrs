use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum PluginType {
    DocFragments,
    Callback,
    Connection,
    Shell,
}

impl fmt::Display for PluginType {
    /// Provide a string representation of each variant
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let variant_name = match self {
            PluginType::DocFragments => "DocFragments",
            PluginType::Callback => "Callback",
            PluginType::Connection => "Connection",
            PluginType::Shell => "Shell",
        };
        write!(f, "{}", variant_name)
    }
}

impl PluginType {
    /// Generate a stable hash-based `u32` identifier using the `Display` value
    pub fn id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.to_string().hash(&mut hasher); // Use the string representation in hashing
        hasher.finish()
    }

    /// Convert a `u64` back into a `PluginType`, matching against hashed IDs
    pub fn from_u64(n: u64) -> Option<PluginType> {
        [
            PluginType::DocFragments,
            PluginType::Callback,
            PluginType::Connection,
            PluginType::Shell,
        ]
        .into_iter()
        .find(|variant| variant.id() == n)
    }
}
