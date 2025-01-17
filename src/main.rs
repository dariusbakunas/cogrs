mod inventory;

use log::{debug, error, info, log_enabled, warn, Level};
use std::collections::HashMap;
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use openssh::{KnownHosts, Session};
use serde_yaml::{self};
use anyhow::{Result};
use crate::inventory::{filter_hosts, load_inventory, HostGroup};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// outputs a list of matching hosts; does not execute anything else
    #[arg(short, long, action)]
    list_hosts: bool,

    /// the action's options in space separated k=v format: -a 'opt1=val1 opt2=val2' or a json string: -a '{"opt1": "val1", "opt2": "val2"}'
    #[arg(short, long)]
    args: Option<String>,

    /// name of the action to execute
    #[arg(short, long, default_value = "shell")]
    module_name: String,

    /// host pattern
    pattern: String,

    /// specify inventory host path
    #[arg(short, long, value_name = "FILE")]
    inventory: Option<PathBuf>,

    #[command(subcommand)]
    cmd: Option<Commands>
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show CogRS inventory information
    Inventory {
        /// specify inventory host path
        #[arg(short, long, value_name = "FILE")]
        inventory: PathBuf
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let cli = Cli::parse();
    let mut hosts: Option<Vec<String>> = None;

    match cli.inventory {
        Some(inventory) => {
            let inventory = load_inventory(&inventory);
            hosts = Some(filter_hosts(&inventory, &cli.pattern));
        }
        None => {
            warn!("no inventory was parsed, only implicit localhost is available");
        }
    }

    if cli.list_hosts {
        match hosts {
            Some(hosts) => {
                for host in hosts {
                    println!("{}", host);
                }
            },
            None => {}
        }
    } else {
        if cli.module_name == "shell" {
            let args = cli.args.unwrap();
            match hosts {
                Some(hosts) => {
                    for host in &hosts {
                        info!("executing on host: {}", host);
                        let session = Session::connect_mux(format!("ssh://{}:22", host), KnownHosts::Strict)
                            .await?;

                        let mut cmd = session.command("bash");
                        cmd.arg("-c");
                        cmd.arg(&args);

                        let output = cmd.output().await.unwrap();
                        eprintln!(
                            "{}",
                            String::from_utf8(output.stdout).expect("server output was not valid UTF-8")
                        );
                        session.close().await.unwrap();
                    }
                },
                None => {
                    // TODO
                    let mut cmd = std::process::Command::new("bash");
                    cmd.arg("-c");
                    cmd.arg(args);
                    let out = cmd.output().expect("failed to execute process");
                    print!("{}", String::from_utf8_lossy(&out.stdout));
                }
            }

        } else {
            warn!("module '{}' not implemented", cli.module_name);
        }
    }

    match &cli.cmd {
        Some(Commands::Inventory { inventory }) => {
            let f = std::fs::File::open(inventory).expect("Could not open inventory file.");
            let deser: HashMap<String, HostGroup> = serde_yaml::from_reader(f).expect("Could not read inventory file.");
            println!("{:#?}", deser);
        }
        None => {}
    }

    Ok(())
}
