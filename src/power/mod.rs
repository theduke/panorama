//! Read power supply information from the system.
//! See https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-power

use std::time::SystemTime;

use anyhow::Context;
use futures::StreamExt;

use crate::notify::Notifier;

use self::{cfg::PowerConfig, system::PowerSupplyType};

pub mod cfg;
pub mod system;

const ALERT_GROUP_BATTERY: &str = "panorama.battery_status";

pub struct PowerManager {
    config: PowerConfig,
    notifier: Notifier,
    battery_phase: Option<PhaseTransition>,
    mode: PowerMode,
}

impl PowerManager {
    pub async fn run(config: PowerConfig, notifier: Notifier) -> Result<(), anyhow::Error> {
        let manager = Self::new(config, notifier)?;
        tokio::task::spawn_local(async move { manager.run_loop().await })
            .await
            .context("PowerManager failed")??;

        Ok(())
    }

    fn new(config: PowerConfig, notifier: Notifier) -> Result<Self, anyhow::Error> {
        let config = config.validate()?;
        Ok(Self {
            config,
            notifier,
            battery_phase: None,
            mode: PowerMode::PluggedIn,
        })
    }

    async fn run_loop(mut self) -> Result<(), anyhow::Error> {
        // Listen to udev power_supply events to learn of state changes as
        // quickly as possible.
        let builder = tokio_udev::MonitorBuilder::new()
            .expect("Couldn't create builder")
            .match_subsystem("power_supply")
            .context("could not create power_supply filter")?;

        let mut stream: tokio_udev::AsyncMonitorSocket = builder
            .listen()
            .context("Couldn't listen on udev socket")?
            .try_into()
            .context("could not create udev monitor socket")?;

        eprintln!("reading udev event stream...");

        loop {
            self.tick().await?;

            let timeout = tokio::time::sleep(std::time::Duration::from_secs(
                self.config.refresh_interval_seconds,
            ));

            // Wait for a tick timeout, or a udev power supply event.
            tokio::select! {
                _ = timeout => {},
                _ev = stream.next() => {
                    // No need to actually interpret the udev event data,
                    // we re-parse the /sys/ data anyway.
                    // udev is just used to get fast notifications.
                    // TODO: maybe batch up events to prevent constant re-computations?
                    // (unplugging will trigger two notifications, one for the
                    // AC and one for the battery)
                }
            }
        }
    }

    async fn tick(&mut self) -> Result<(), anyhow::Error> {
        let supplies = tokio::task::spawn_blocking(|| system::read_all_supplies()).await??;

        let ac = supplies.iter().find_map(|s| s.kind.as_main());
        let ac_online = ac.map(|s| s.online).unwrap_or(false);

        // TODO: support multiple batteries.
        let battery_opt = supplies
            .iter()
            .find_map(|s| match &s.kind {
                PowerSupplyType::Battery(b) => Some((s, b)),
                PowerSupplyType::Main(_) => None,
            })
            .map(|x| x.1);

        let mut variables = std::collections::HashMap::new();
        if let Some(bat) = &battery_opt {
            variables.insert("capacity".to_string(), bat.capacity.to_string());
        }

        if ac_online && self.mode != PowerMode::PluggedIn {
            tracing::trace!("power mode changed to plugged in");
            self.mode = PowerMode::PluggedIn;
            if let Some(alert) = &self.config.alert_battery_deactivated {
                let full = alert.prepare(ALERT_GROUP_BATTERY.to_string(), variables.clone());
                self.notifier.notify(full).await?;
            }
        } else if !ac_online && self.mode != PowerMode::Battery {
            tracing::trace!("power mode changed to battery");
            self.mode = PowerMode::Battery;

            if let Some(alert) = &self.config.alert_battery_activated {
                let full = alert.prepare(ALERT_GROUP_BATTERY.to_string(), variables.clone());
                self.notifier.notify(full).await?;
            }
        }

        if self.mode == PowerMode::Battery {
            let bat = battery_opt.as_ref().unwrap();

            let phase = self
                .config
                .phases
                .iter()
                .find(|p| bat.capacity >= p.from && bat.capacity <= p.to);

            match (phase, &mut self.battery_phase) {
                (Some(new), Some(status)) => {
                    if new.name != status.name {
                        status.enter(&new.name);

                        if let Some(alert) = &new.alert {
                            let full = alert.prepare(ALERT_GROUP_BATTERY.to_string(), variables);
                            self.notifier.notify(full).await?;
                        }
                    } else {
                        let now = SystemTime::now();
                        let last_notified_at = status.last_notified_at.unwrap_or(now);

                        let elapsed = now.duration_since(last_notified_at).unwrap();

                        if let Some(alert) = &new.alert {
                            if let Some(repeat_after) = alert.repeat_after_seconds {
                                if elapsed.as_secs() >= repeat_after {
                                    let full = alert.prepare(
                                        ALERT_GROUP_BATTERY.to_string(),
                                        variables.clone(),
                                    );
                                    self.notifier.notify(full).await?;
                                    status.last_notified_at = Some(now);
                                }
                            }
                        }
                    }
                }
                (Some(new), None) => {
                    let status = PhaseTransition {
                        name: new.name.clone(),
                        entered_at: SystemTime::now(),
                        last_notified_at: None,
                    };
                    self.battery_phase = Some(status);

                    if let Some(alert) = &new.alert {
                        let full =
                            alert.prepare(ALERT_GROUP_BATTERY.to_string(), variables.clone());
                        self.notifier.notify(full).await?;
                    }
                }
                (None, Some(_status)) => {
                    self.battery_phase = None;
                }
                _ => {}
            }
        }

        tracing::trace!("power tick");

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct PhaseTransition {
    name: String,
    entered_at: SystemTime,
    last_notified_at: Option<SystemTime>,
}

impl PhaseTransition {
    fn enter(&mut self, name: &str) {
        self.name = name.to_string();
        self.entered_at = SystemTime::now();
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PowerMode {
    Battery,
    PluggedIn,
}
