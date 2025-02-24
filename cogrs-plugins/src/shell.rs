pub trait ShellPlugin: Send + Sync {
    fn quote(&self, value: &str) -> String;
    fn pwd(&self) -> String;
    fn expand_user(&self, home_path: &str, username: &str) -> String;
    fn mk_temp(&self, base_path: &str, system: bool, mode: u32, tmp_dir: Option<&str>) -> String;
}

#[macro_export]
macro_rules! create_shell_plugin {
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
    };
}
