use anyhow::Result;
use clap::Parser;
use cogrs::cli::Cli;
use cogrs::inventory::manager;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    run()
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let inventory = cli.inventory.as_ref();

    let mut manager = manager::InventoryManager::new();
    manager.parse_sources(inventory)?;

    if cli.list_hosts {
        let mut hosts: Vec<String> = manager
            .filter_hosts(cli.pattern.as_str(), cli.limit.as_deref())?
            .iter()
            .map(|h| h.name.to_string())
            .collect();
        hosts.sort();
        for host in hosts {
            println!("{host}");
        }
        Ok(())
    } else {
        todo!();
    }
}
