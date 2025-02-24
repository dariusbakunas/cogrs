use cogrs_plugins::create_shell_plugin;
use cogrs_plugins::plugin_type::PluginType;
use cogrs_plugins::shell::ShellPlugin;

create_shell_plugin!(Sh, "sh", {
   compatible_shells: Vec<String>,
});

impl ShellPlugin for Sh {
    fn quote(&self, value: &str) -> String {
        format!("'{}'", value)
    }

    fn pwd(&self) -> String {
        todo!()
    }

    fn expand_user(&self, home_path: &str, username: &str) -> String {
        todo!()
    }

    fn mk_temp(&self, base_path: &str, system: bool, mode: u32, tmp_dir: Option<&str>) -> String {
        todo!()
    }
}
