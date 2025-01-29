use crate::ssh::execute_on_host;
use cogrs::cli::Cli;
use log::{error, warn};

#[allow(dead_code)]
pub async fn handle_module_execution(module: &str, cli: &Cli, hosts: Option<Vec<String>>) {
    if module == "shell" {
        let args = cli.args.clone().unwrap();
        match hosts {
            Some(hosts) => {
                for host in &hosts {
                    if let Err(e) = execute_on_host(host, args.as_str()).await {
                        error!("failed to execute on host '{}': {}", host, e);
                    }
                }
            }
            None => {
                let mut cmd = std::process::Command::new("bash");
                cmd.arg("-c");
                cmd.arg(args);
                let out = cmd.output().expect("failed to execute process");
                print!("{}", String::from_utf8_lossy(&out.stdout));
            }
        }
    } else {
        warn!("module '{}' not implemented", module);
    }
}
