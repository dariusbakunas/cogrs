use anyhow::Result;
use clap::Parser;
use cogrs::cli::Cli;
use cogrs_core::adhoc::{AdHoc, AdHocOptions};
use cogrs_core::inventory::manager;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    run().await?;

    Ok(())
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let inventory = cli.inventory.as_deref();
    let playbook_dir = cli.resolved_playbook_dir();

    let mut manager = manager::InventoryManager::new(&playbook_dir);
    manager.parse_sources(inventory)?;
    let pattern = cli.pattern.as_str();

    if cli.list_hosts {
        let hosts = manager.filter_hosts(cli.pattern.as_str(), cli.limit.as_deref())?;
        // ansible seems to ignore everything else if --list-hosts is specified?
        for host in hosts {
            println!("{}", host.name());
        }
    } else if let Some(module_name) = cli.module_name {
        let options = AdHocOptions {
            forks: cli.forks,
            poll_interval: Some(cli.poll_interval),
            task_timeout: cli.task_timeout,
            async_val: cli.async_val,
            one_line: cli.one_line,
        };

        AdHoc::run(
            pattern,
            cli.limit.as_deref(),
            &module_name,
            cli.args,
            &manager,
            &options,
        )
        .await?;
    } else {
        todo!()
    }

    Ok(())
}
