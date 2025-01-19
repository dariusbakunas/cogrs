mod cli;
mod inventory;
mod modules;
mod playbook;
mod ssh;


use anyhow::Result;
use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    let cli = Cli::parse();
    cogrs::run(cli.inventory.as_ref())
}
