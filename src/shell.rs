use anyhow::{Result};

pub async fn execute_on_host(host: &str, args: &str) -> Result<()> {
    use openssh::{Session, KnownHosts};
    use log::info;

    info!("executing on host: {}", host);
    let session = Session::connect_mux(format!("ssh://{}:22", host), KnownHosts::Strict).await?;

    let mut cmd = session.command("bash");
    cmd.arg("-c");
    cmd.arg(args);

    let output = cmd.output().await?;
    eprintln!(
        "{}",
        String::from_utf8(output.stdout).expect("server output was not valid UTF-8")
    );
    session.close().await?;

    Ok(())
}