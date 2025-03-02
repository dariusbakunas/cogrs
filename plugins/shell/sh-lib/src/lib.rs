use anyhow::Result;
use cogrs_plugins::create_shell_plugin;
use cogrs_plugins::shell::ShellPlugin;

create_shell_plugin!(Sh, {
   compatible_shells: Vec<String>,
});

impl ShellPlugin for Sh {
    fn shell_and(&self) -> String {
        String::from("&&")
    }

    fn quote(&self, value: &str) -> String {
        format!("'{}'", value)
    }

    fn pwd(&self) -> String {
        todo!()
    }

    fn mk_temp(&self, base_path: &str, system: bool, mode: u32, tmp_dir: Option<&str>) -> String {
        todo!()
    }
}
