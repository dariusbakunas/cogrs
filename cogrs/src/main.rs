use anyhow::{anyhow, Context, Result};
use clap::Parser;
use cogrs::cli::Cli;
use cogrs_core::adhoc::AdHoc;
use cogrs_core::inventory::manager;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    run()
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let inventory = cli.inventory.as_deref();

    let mut manager = manager::InventoryManager::new();
    manager.parse_sources(inventory)?;

    let hosts = manager.filter_hosts(cli.pattern.as_str(), cli.limit.as_deref())?;

    if cli.list_hosts {
        // ansible seems to ignore everything else if --list-hosts is specified?
        for host in hosts {
            println!("{}", host.name);
        }
    } else if let Some(module_name) = cli.module_name {
        let args = cli
            .args
            .with_context(|| anyhow!("No argument passed to {module_name} module"))?;
        AdHoc::run(
            &module_name,
            &args,
            &hosts,
            cli.forks,
            Some(cli.poll_interval),
            cli.task_timeout,
            cli.async_val,
            cli.one_line,
        )?;
    } else {
        todo!()
    }

    Ok(())
}
