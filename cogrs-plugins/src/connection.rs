use crate::shell::ShellPlugin;
use anyhow::{Context, Result};
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
    fn initialize(&mut self, shell: Box<dyn ShellPlugin>, parameters: &str) -> Result<()>;

    fn validate_parameters(&self, parameters: &str) -> Result<()> {
        let parsed_params: Value = serde_json::from_str(parameters)
            .context("Failed to parse parameters into JSON value")?;
        validate_input(self.schema(), &parsed_params)
    }

    fn schema(&self) -> &'static str;
}

#[macro_export]
macro_rules! create_connection_plugin {
    ($plugin_name:ident, $plugin_name_str: expr, { $($field_name:ident: $field_type:ty),* $(,)? }) => {
        pub struct $plugin_name {
            $(pub $field_name: $field_type),*
        }

        impl Default for $plugin_name {
            fn default() -> Self {
                Self {
                    $($field_name: Default::default()),*
                }
            }
        }

        impl $plugin_name {
            pub fn new($($field_name: $field_type),*) -> Self {
                Self {
                    $($field_name),*
                }
            }
        }

        #[no_mangle]
        pub fn create_plugin() -> Box<dyn ConnectionPlugin> {
            Box::new($plugin_name::default()
)
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
