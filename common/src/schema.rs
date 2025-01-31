#[macro_export]
macro_rules! define_schema {
    ($schema:expr) => {
        pub const SCHEMA: &str = $schema;
    };
}
