#[macro_export]
macro_rules! define_schema {
    ($struct_name:ident, $schema:expr) => {
        pub const SCHEMA: &str = $schema;
        use std::str::FromStr;

        impl std::str::FromStr for $struct_name {
            type Err = serde_json::Error;

            fn from_str(json: &str) -> Result<Self, Self::Err> {
                serde_json::from_str(json)
            }
        }
    };
}
