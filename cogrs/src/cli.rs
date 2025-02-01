use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version = env!("APP_VERSION"), about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// outputs a list of matching hosts; does not execute anything else
    #[arg(long, action)]
    pub list_hosts: bool,

    /// the action's options as json string: -a '{"opt1": "val1", "opt2": "val2"}'
    /// should match module schema, to get schema run: `module_name --schema`
    #[arg(short, long)]
    pub args: Option<String>,

    /// name of the action to execute
    #[arg(short, long, group = "action")]
    pub module_name: Option<String>,

    /// further limit selected hosts to an additional pattern
    #[arg(short, long, value_name = "SUBSET")]
    pub limit: Option<String>,

    /// host pattern
    pub pattern: String,

    /// specify inventory host path
    #[arg(short, long)]
    pub inventory: Option<Vec<String>>,

    /// specify playbook you want to run
    #[arg(short, long, value_name = "FILE", group = "action")]
    pub playbook: Option<PathBuf>,
}
