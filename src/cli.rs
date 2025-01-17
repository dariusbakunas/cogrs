use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// outputs a list of matching hosts; does not execute anything else
    #[arg(short, long, action)]
    pub list_hosts: bool,

    /// the action's options in space separated k=v format: -a 'opt1=val1 opt2=val2' or a json string: -a '{"opt1": "val1", "opt2": "val2"}'
    #[arg(short, long)]
    pub args: Option<String>,

    /// name of the action to execute
    #[arg(short, long, group = "action")]
    pub module_name: Option<String>,

    /// host pattern
    pub pattern: String,

    /// specify inventory host path
    #[arg(short, long, value_name = "FILE")]
    pub inventory: Option<PathBuf>,

    /// specify playbook you want to run
    #[arg(short, long, value_name = "FILE", group = "action")]
    pub playbook: Option<PathBuf>,
}
