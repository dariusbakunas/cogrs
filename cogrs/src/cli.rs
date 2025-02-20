use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version = env!("APP_VERSION"), about, long_about = None)]
pub struct Cli {
    #[arg(long, action)]
    /// outputs a list of matching hosts; does not execute anything else
    pub list_hosts: bool,

    #[arg(short, long, value_name = "CONNECTION", default_value = "ssh")]
    /// connection type to use
    pub connection: String,

    #[arg(short = 'C', long, action)]
    /// don't make any changes; instead, try to predict some of the changes that may occur
    pub check: bool,

    #[arg(short = 'D', long, action)]
    /// when changing (small) files and templates, show the differences in those files; works great with --check
    pub diff: bool,

    #[arg(short, long)]
    /// the action's options as json string: -a '{"opt1": "val1", "opt2": "val2"}'
    /// should match module schema, to get schema run: `module_name --schema`
    pub args: Option<String>,

    #[arg(short, long, default_value = "5")]
    /// specify number of parallel processes to use
    pub forks: u32,

    #[arg(short, long, group = "action")]
    /// name of the action to execute
    pub module_name: Option<String>,

    #[arg(short, long, value_name = "SUBSET")]
    /// further limit selected hosts to an additional pattern
    pub limit: Option<String>,

    #[arg(long, value_name = "SECONDS")]
    /// set task timeout limit in seconds, must be positive integer
    pub task_timeout: Option<u64>,

    #[arg(long = "timeout", value_name = "TIMEOUT")]
    /// override the connection timeout in seconds (default depends on connection)
    pub connection_timeout: Option<u64>,

    #[arg(long = "private-key", value_name = "PRIVATE_KEY_FILE")]
    /// use this file to authenticate the connection
    pub private_key_file: Option<PathBuf>,

    #[arg(short = 'B', long, value_name = "SECONDS")]
    /// run asynchronously, failing after X seconds
    pub async_val: Option<u64>,

    #[arg(long, value_name = "BASEDIR", value_parser, default_value = ".")]
    pub playbook_dir: PathBuf,

    #[arg(
        short = 'P',
        long = "poll",
        value_name = "SECONDS",
        default_value = "15"
    )]
    /// set the poll interval if using -B
    pub poll_interval: u64,

    #[arg(short, long)]
    pub one_line: bool,

    /// host pattern
    pub pattern: String,

    #[arg(short, long)]
    /// specify inventory host path
    pub inventory: Option<Vec<String>>,

    #[arg(short, long, value_name = "FILE", group = "action")]
    /// specify playbook you want to run
    pub playbook: Option<PathBuf>,
}

impl Cli {
    /// Resolves the playbook_dir to an absolute path
    pub fn resolved_playbook_dir(&self) -> PathBuf {
        fs::canonicalize(&self.playbook_dir).unwrap_or_else(|_| self.playbook_dir.clone())
    }
}
