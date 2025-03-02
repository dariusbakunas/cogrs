use anyhow::Result;
use rand::Rng;
use regex::Regex;
use shlex::Quoter;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

pub trait ShellPlugin: Send + Sync {
    fn shell_and(&self) -> String;
    fn quote(&self, value: &str) -> String;
    fn pwd(&self) -> String;
    fn expand_user(&self, home_path: &str, username: Option<&str>) -> Result<String> {
        let mut path = home_path.to_owned();
        let user_home_path_re = Regex::new(r"^~[_.A-Za-z0-9][-_.A-Za-z0-9]*$")?;

        if path != "~" {
            if !user_home_path_re.is_match(&path) {
                // Escape the potentially unsafe path
                path = Quoter::new()
                    .allow_nul(true)
                    .quote(&path)
                    .map(|s| s.to_string())?;
            }
        } else if let Some(user) = username {
            // Append the username if present
            path.push_str(user);
        }

        Ok(path)
    }
    fn mk_temp(&self, base_path: &str, system: bool, mode: u32, tmp_dir: Option<&str>) -> String;
    fn generate_temp_dir_name(&self) -> String {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis(); // Get milliseconds for higher precision

        // Get the process ID
        let process_id = process::id();

        // Generate a random number within the range of 48-bit unsigned integer
        let random_number = rand::rng().random_range(0..(1u64 << 48));

        // Format the final string
        format!(
            "cogrs-tmp-{}-{}-{}",
            current_time, process_id, random_number
        )
    }
}

#[macro_export]
macro_rules! create_shell_plugin {
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

        impl $plugin_name {
            pub fn new($($field_name: $field_type),*) -> Self {
                Self {
                    $($field_name),*
                }
            }
        }
    };
}

/// Macro for generating plugin metadata and FFI exports
#[macro_export]
macro_rules! create_shell_plugin_exports {
    (
        $plugin_name:ident, // Struct name of the plugin
        $plugin_name_str:expr, // Plugin's name as a string
        $versions:expr // Supported versions (HashMap)
    ) => {
        use cogrs_plugins::plugin_type::PluginType;
        use cogrs_plugins::shell::ShellPlugin;

        #[no_mangle]
        pub fn create_plugin() -> Box<dyn ShellPlugin> {
            Box::new($plugin_name::default())
        }

        #[no_mangle]
        pub extern "C" fn plugin_type() -> u64 {
            PluginType::Shell.id()
        }

        #[no_mangle]
        pub extern "C" fn plugin_name() -> *const std::os::raw::c_char {
            // Ensure the string is null-terminated explicitly
            concat!($plugin_name_str, "\0").as_ptr() as *const std::os::raw::c_char
        }

        #[no_mangle]
        pub extern "C" fn cogrs_versions() -> *const std::os::raw::c_char {
            let versions = serde_json::to_string(&$versions).unwrap();

            std::ffi::CString::new(versions).unwrap().into_raw()
        }
    };
}
