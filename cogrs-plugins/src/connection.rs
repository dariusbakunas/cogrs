use anyhow::{Context, Result};
use async_trait::async_trait;
use cogrs_schema::validation::validate_input;
use serde_json::Value;

pub struct CommandOutput {
    stdout: String,
    stderr: String,
    rc: i32,
}

impl CommandOutput {
    pub fn new(stdout: String, stderr: String, rc: i32) -> Self {
        CommandOutput { stdout, stderr, rc }
    }
}

#[async_trait]
pub trait ConnectionPlugin: Send + Sync {
    fn do_become(&self) -> bool;

    fn become_user(&self) -> Option<String>;

    /// Checks if the plugin is currently connected.
    fn connected(&self) -> bool;

    /// Retrieves the remote architecture
    fn get_remote_architecture(&self) -> Result<String>;

    /// Executes a command on the remote system.
    async fn exec_command(&self, command: &str) -> Result<CommandOutput>;

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
        let parsed_params: Value = serde_json::from_str(parameters)
            .context("Failed to parse parameters into JSON value")?;
        validate_input(self.schema(), &parsed_params)
    }

    fn schema(&self) -> &'static str;
    fn remote_user(&self) -> String;
}

#[macro_export]
macro_rules! create_connection_plugin {
    ($plugin_name:ident, { $($field_name:ident: $field_type:ty),* $(,)? }) => {
        use serde_json::json;

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
    };
}

/// Macro for generating plugin metadata and FFI exports
#[macro_export]
macro_rules! create_connection_plugin_exports {
    (
        $plugin_name:ident, // Struct name of the plugin
        $plugin_name_str:expr, // Plugin's name as a string
        $versions:expr // Supported versions (HashMap)
    ) => {
        use cogrs_plugins::connection::ConnectionPlugin;
        use cogrs_plugins::plugin_type::PluginType;

        #[no_mangle]
        pub fn create_plugin() -> Box<dyn ConnectionPlugin> {
            // Instantiate the plugin dynamically
            Box::new($plugin_name::default())
        }

        #[no_mangle]
        pub extern "C" fn plugin_type() -> u64 {
            // Returns the type of the plugin (hardcoded or otherwise)
            PluginType::Connection.id()
        }

        #[no_mangle]
        pub extern "C" fn plugin_name() -> *const std::os::raw::c_char {
            // Returns a null-terminated string of the plugin name
            concat!($plugin_name_str, "\0").as_ptr() as *const std::os::raw::c_char
        }

        #[no_mangle]
        pub extern "C" fn cogrs_versions() -> *const std::os::raw::c_char {
            // Serializes and returns the plugin's supported versions as JSON
            let versions = serde_json::to_string(&$versions).unwrap();
            std::ffi::CString::new(versions).unwrap().into_raw()
        }
    };
}
