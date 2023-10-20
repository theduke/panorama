//! Read power supply information from the system.
//! See https://www.kernel.org/doc/Documentation/ABI/testing/sysfs-class-power

use std::time::SystemTime;

use crate::{notify::Notifier, ResultCallback};

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
    pub fn start(
        config: PowerConfig,
        on_failure: ResultCallback,
        notifier: Notifier,
    ) -> Result<(), anyhow::Error> {
        let manager = Self::new(config, notifier)?;
        std::thread::spawn(move || {
            let res = manager.run();
            on_failure(res);
        });

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

    fn run(mut self) -> Result<(), anyhow::Error> {
        loop {
            self.tick()?;
            std::thread::sleep(std::time::Duration::from_secs(
                self.config.refresh_interval_seconds,
            ));
        }
    }

    fn tick(&mut self) -> Result<(), anyhow::Error> {
        let supplies = system::read_all_supplies()?;

        let ac = supplies.iter().find_map(|s| s.kind.as_main());
        let ac_online = ac.map(|s| s.online).unwrap_or(false);

        if ac_online && self.mode != PowerMode::PluggedIn {
            tracing::trace!("power mode changed to plugged in");
            self.mode = PowerMode::PluggedIn;
            if let Some(alert) = &self.config.alert_battery_deactivated {
                self.notifier.notify(alert, Some(ALERT_GROUP_BATTERY))?;
            }
        } else if !ac_online && self.mode != PowerMode::Battery {
            tracing::trace!("power mode changed to battery");
            self.mode = PowerMode::Battery;

            if let Some(alert) = &self.config.alert_battery_activated {
                self.notifier.notify(alert, Some(ALERT_GROUP_BATTERY))?;
            }
        }

        if self.mode == PowerMode::Battery {
            // TODO: support multiple batteries.
            let (_, bat) = supplies
                .iter()
                .find_map(|s| match &s.kind {
                    PowerSupplyType::Battery(b) => Some((s, b)),
                    PowerSupplyType::Main(_) => None,
                })
                .unwrap();

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
                            self.notifier.notify(alert, Some(ALERT_GROUP_BATTERY))?;
                        }
                    } else {
                        let now = SystemTime::now();
                        let last_notified_at = status.last_notified_at.unwrap_or(now);

                        let elapsed = now.duration_since(last_notified_at).unwrap();

                        if let Some(alert) = &new.alert {
                            if let Some(repeat_after) = alert.repeat_after_seconds {
                                if elapsed.as_secs() >= repeat_after {
                                    self.notifier.notify(alert, Some(ALERT_GROUP_BATTERY))?;
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
                        self.notifier.notify(alert, Some(ALERT_GROUP_BATTERY))?;
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
