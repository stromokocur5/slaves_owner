use std::process::Stdio;

use anyhow::{anyhow, Result};

#[tokio::main]
async fn main() -> Result<()> {
    if !running_as_root() {
        return Err(anyhow!("Must run as root"));
    }
    install().await?;
    Ok(())
}

fn running_as_root() -> bool {
    use users::{get_current_uid, get_user_by_uid};

    let user = get_user_by_uid(get_current_uid()).unwrap();
    match user.name().to_str() {
        Some("root") => true,
        _ => false,
    }
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

async fn service() {}
async fn sound() {}
async fn video() {}
