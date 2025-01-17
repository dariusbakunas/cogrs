mod inventory;

use std::collections::HashMap;
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use serde_yaml::{self};
use crate::inventory::{filter_hosts, load_inventory, HostGroup};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// outputs a list of matching hosts; does not execute anything else
    #[arg(short, long, action)]
    list_hosts: bool,

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
    /// Show AutoRS inventory information
    Inventory {
        /// specify inventory host path
        #[arg(short, long, value_name = "FILE")]
        inventory: PathBuf
    }
}

fn main() {
    let cli = Cli::parse();

    if cli.list_hosts {
        match cli.inventory {
            Some(inventory) => {
                let deser = load_inventory(&inventory);
                let hosts = filter_hosts(&deser, &cli.pattern);

                for host in hosts {
                    println!("{}", host);
                }
            }
            None => {}
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
}
