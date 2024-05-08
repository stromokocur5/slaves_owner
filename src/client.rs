use std::process::Stdio;

use anyhow::{anyhow, Result};

use slaves_owner::running_as_root;

#[tokio::main]
async fn main() -> Result<()> {
    if !running_as_root() {
        return Err(anyhow!("Must run as root"));
    }
    Ok(())
}

async fn install() -> Result<()> {
    let curl = std::process::Command::new("curl")
        .args([
            "-fsSL",
            "https://raw.githubusercontent.com/sigoden/upt/main/install.sh",
        ])
        .stdout(Stdio::piped())
        .spawn()?;
    let mut sh = std::process::Command::new("sh")
        .args(["-s", "--", "--to", "/usr/local/bin"])
        .stdin(curl.stdout.unwrap())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    sh.wait()?;
    let required_packages = &["mpv", "alsa-tools"];
    for i in required_packages {
        upt_install(i).await?;
    }
    Ok(())
}

async fn upt_install(package: &str) -> Result<()> {
    let mut upt = std::process::Command::new("upt")
        .args(["install", "-y", package])
        .stdout(Stdio::null())
        .spawn()?;
    upt.wait()?;
    Ok(())
}

async fn service() -> Result<()> {
    use tokio::fs;
    let client_service = include_str!("../slaves_owner_client.service");
    fs::write("/etc/systemd/system/ssc.service", client_service).await?;
    let mut systemctl = std::process::Command::new("systemctl")
        .args(["enable", "--now", "ssc"])
        .stdout(Stdio::null())
        .spawn()?;
    systemctl.wait()?;
    Ok(())
}
async fn always_on_audio() -> Result<()> {
    let mut amixer = std::process::Command::new("amixer")
        .args(["sset", "Master", "unmute"])
        .stdout(Stdio::null())
        .spawn()?;
    amixer.wait()?;
    let mut amixer = std::process::Command::new("amixer")
        .args(["set", "Master", "40%"])
        .stdout(Stdio::null())
        .spawn()?;
    amixer.wait()?;
    Ok(())
}
async fn record_audio() -> Result<()> {
    Ok(())
}
async fn record_video() -> Result<()> {
    use tokio_linux_video::Device;
    let mut devs = Device::list().await?;
    while let Some(path) = devs.fetch_next().await? {
        let dev = Device::open(&path).await?;

        let caps = dev.capabilities().await?;

        println!("path: {}, {caps}", path.display());
    }
    Ok(())
}
