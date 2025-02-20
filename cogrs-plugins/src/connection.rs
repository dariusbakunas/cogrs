use anyhow::Result;

pub trait ConnectionPlugin: Send + Sync {
    fn connected(&self) -> bool;
    fn get_remote_architecture(&self) -> Result<String>;
    fn exec_command(&self, command: &str) -> Result<String>;
    fn put_file(&self, source_path: &str, dest_path: &str) -> Result<()>;
    fn fetch_file(&self, source_path: &str, dest_path: &str) -> Result<()>;
    fn close(&self);
    fn connect(&self) -> Result<()>;
}
