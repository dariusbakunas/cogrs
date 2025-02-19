use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version = env!("APP_VERSION"), about, long_about = None)]
pub struct Cli {
    /// outputs a list of matching hosts; does not execute anything else
    #[arg(long, action)]
    pub list_hosts: bool,

    #[arg(short, long, value_name = "CONNECTION", default_value = "ssh")]
    pub connection: String,

    /// the action's options as json string: -a '{"opt1": "val1", "opt2": "val2"}'
    /// should match module schema, to get schema run: `module_name --schema`
    #[arg(short, long)]
    pub args: Option<String>,

    #[arg(short, long, default_value = "5")]
    pub forks: u32,

    /// name of the action to execute
    #[arg(short, long, group = "action")]
    pub module_name: Option<String>,

    /// further limit selected hosts to an additional pattern
    #[arg(short, long, value_name = "SUBSET")]
    pub limit: Option<String>,

    #[arg(long, value_name = "SECONDS")]
    pub task_timeout: Option<u64>,

    #[arg(short = 'B', long, value_name = "SECONDS")]
    pub async_val: Option<u64>,

    #[arg(long, value_name = "BASEDIR", value_parser, default_value = ".")]
    pub playbook_dir: PathBuf,

    #[arg(
        short = 'P',
        long = "poll",
        value_name = "SECONDS",
        default_value = "15"
    )]
    pub poll_interval: u64,

    #[arg(short, long)]
    pub one_line: bool,

    /// host pattern
    pub pattern: String,

    /// specify inventory host path
    #[arg(short, long)]
    pub inventory: Option<Vec<String>>,

    /// specify playbook you want to run
    #[arg(short, long, value_name = "FILE", group = "action")]
    pub playbook: Option<PathBuf>,
}

impl Cli {
    /// Resolves the playbook_dir to an absolute path
    pub fn resolved_playbook_dir(&self) -> PathBuf {
        fs::canonicalize(&self.playbook_dir).unwrap_or_else(|_| self.playbook_dir.clone())
    }
}
