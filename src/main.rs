use std::collections::HashMap;
use std::iter::Map;
use std::path::PathBuf;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_yaml::{self};

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

#[derive(Debug, Serialize, Deserialize)]
struct HostGroup {
    hosts: Option<HashMap<String, Host>>
}

#[derive(Debug, Serialize, Deserialize)]
struct Host {
}

fn main() {
    let cli = Cli::parse();

    if cli.list_hosts {
        match cli.inventory {
            Some(inventory) => {
                let f = std::fs::File::open(inventory).expect("Could not open inventory file.");
                let deser: HashMap<String, HostGroup> = serde_yaml::from_reader(f).expect("Could not read inventory file.");

                let mut hosts: Vec<String> = Vec::new();

                if cli.pattern == "all" {
                    for (_, value) in deser.into_iter() {
                        match value.hosts {
                            Some(h) => {
                                hosts.extend(h.keys().cloned());
                            }
                            None => {}
                        }
                    }
                } else {
                    let patterns: Vec<&str> = cli.pattern
                        .split([':', ',']) // Split by ':' or ','
                        .collect();

                    for (group_name, value) in deser.into_iter() {
                        if patterns.contains(&group_name.as_str()) {
                            match &value.hosts {
                                Some(h) => {
                                    hosts.extend(h.keys().cloned());
                                }
                                None => {}
                            }
                        } else {
                            match &value.hosts {
                                Some(h) => {
                                    hosts.extend(
                                        h.keys()
                                            .filter(|key| patterns.contains(&key.as_str()))
                                            .cloned()
                                    );
                                }
                                None => {}
                            }
                        }
                    }

                }

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
