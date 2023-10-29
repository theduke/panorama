use anyhow::Context as _;
use futures::StreamExt;
use tokio_udev::{AsyncMonitorSocket, MonitorBuilder};

pub async fn run() -> Result<(), anyhow::Error> {
    let builder = MonitorBuilder::new()
        .expect("Couldn't create builder")
        .match_subsystem_devtype("usb", "usb_device")
        .context("Failed to add filter for USB devices")?
        .match_subsystem("power_supply")
        .context("could not create power_supply filter")?;

    let mut stream: AsyncMonitorSocket = builder
        .listen()
        .context("Couldn't listen on udev socket")?
        .try_into()
        .context("could not create udev monitor socket")?;

    while let Some(res) = stream.next().await {
        let _ev = res.context("failed to read udev event")?;
    }

    Ok(())
}
