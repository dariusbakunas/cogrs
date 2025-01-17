mod cli;
mod inventory;
mod modules;
mod playbook;
mod ssh;

use crate::inventory::{filter_hosts, load_inventory};
use crate::modules::handle_module_execution;
use crate::playbook::load_playbook;
use anyhow::Result;
use clap::Parser;
use cli::Cli;
use log::{error, warn};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let cli = Cli::parse();
    let mut hosts: Option<Vec<String>> = None;

    match cli.inventory {
        Some(ref inventory) => {
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
            }
            None => {}
        }
    } else {
        if let Some(module) = &cli.module_name {
            handle_module_execution(module, &cli, hosts).await;
        } else if let Some(playbook) = &cli.playbook {
            // TODO
            let playbook = load_playbook(playbook);
            println!("{:?}", playbook);
        } else {
            error!(
                "either a module or a playbook must be specified, use --help for more information"
            )
        }
    }

    Ok(())
}
