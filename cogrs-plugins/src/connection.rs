use anyhow::Result;
use cogrs_schema::validation::validate_input;
use serde_json::Value;

pub trait ConnectionPlugin: Send + Sync {
    /// Checks if the plugin is currently connected.
    fn connected(&self) -> bool;

    /// Retrieves the remote architecture
    fn get_remote_architecture(&self) -> Result<String>;

    /// Executes a command on the remote system.
    fn exec_command(&self, command: &str) -> Result<String>;

    /// Uploads a file to the remote system
    fn put_file(&self, source_path: &str, dest_path: &str) -> Result<()>;

    /// Fetches a file from the remote system
    fn fetch_file(&self, source_path: &str, dest_path: &str) -> Result<()>;

    /// Closes the connection.
    fn close(&self);

    /// Establishes a connection.
    fn connect(&self) -> Result<()>;

    /// Initializes the plugin with a set of parameters.
    fn initialize(&mut self, parameters: &str) -> Result<()>;

    fn validate_parameters(&self, parameters: &str) -> Result<()> {
        validate_input(self.schema(), &Value::String(parameters.to_string()))
    }

    fn schema(&self) -> &'static str;
}

#[macro_export]
macro_rules! create_connection_plugin {
    ($plugin_name:ident, $plugin_name_str: expr) => {
        pub struct $plugin_name;
        use std::sync::Arc;

        impl Default for $plugin_name {
            fn default() -> Self {
                Self {}
            }
        }

        #[no_mangle]
        pub fn create_plugin() -> Arc<dyn ConnectionPlugin> {
            Arc::new($plugin_name)
        }

        #[no_mangle]
        pub extern "C" fn plugin_type() -> u64 {
            PluginType::Connection.id()
        }

        #[no_mangle]
        pub extern "C" fn plugin_name() -> *const std::os::raw::c_char {
            // Ensure the string is null-terminated explicitly
            concat!($plugin_name_str, "\0").as_ptr() as *const std::os::raw::c_char
        }
    };
}
