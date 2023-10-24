use serde_derive::{Deserialize, Serialize};

use crate::cfg::{Alert, AlertSeverity};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PowerConfig {
    pub refresh_interval_seconds: u64,
    #[serde(default = "PowerConfig::default_phases")]
    pub phases: Vec<BatteryPhase>,

    #[serde(default = "PowerConfig::default_alert_battery_activated")]
    pub alert_battery_activated: Option<Alert>,
    #[serde(default = "PowerConfig::default_alert_battery_deactivated")]
    pub alert_battery_deactivated: Option<Alert>,
}

impl PowerConfig {
    pub fn validate(mut self) -> Result<Self, anyhow::Error> {
        if self.refresh_interval_seconds == 0 {
            return Err(anyhow::anyhow!(
                "'power.refresh_interval_seconds' must be greater than 0"
            ));
        }

        if self.phases.is_empty() {
            self.phases = Self::default_phases();
        }

        let mut names = std::collections::HashSet::new();
        for (index, phase) in self.phases.iter().enumerate() {
            if !names.insert(&phase.name) {
                anyhow::bail!("Phase '{}' is defined multiple times", phase.name);
            }

            if phase.from > phase.to {
                return Err(anyhow::anyhow!(
                    "Phase '{}' has invalid range: {}..{}",
                    phase.name,
                    phase.from,
                    phase.to
                ));
            }

            if index > 0 {
                let prev = &self.phases[index - 1];
                if phase.from <= prev.to {
                    return Err(anyhow::anyhow!(
                        "Phase '{}' has overlapping range with phase '{}': {}..{}",
                        phase.name,
                        prev.name,
                        phase.from,
                        phase.to
                    ));
                }
            }
        }

        Ok(self)
    }

    pub fn default_refresh_interval_seconds() -> u64 {
        5
    }

    pub fn default_phases() -> Vec<BatteryPhase> {
        vec![
            BatteryPhase {
                name: "almost_empty".to_string(),
                from: 0,
                to: 5,
                alert: Some(Alert {
                    severity: AlertSeverity::Critical,
                    on_startup: true,
                    repeat_after_seconds: Some(60 * 3),
                    summary: "Battery is almost empty! (${capacity}%)".to_string(),
                    message: None,
                    expire_after_seconds: None,
                }),
            },
            BatteryPhase {
                name: "low".to_string(),
                from: 6,
                to: 20,
                alert: Some(Alert {
                    severity: AlertSeverity::Warning,
                    on_startup: true,
                    repeat_after_seconds: Some(60 * 10),
                    summary: "Battery is low! (${capacity}%)".to_string(),
                    message: None,
                    expire_after_seconds: Some(60),
                }),
            },
            BatteryPhase {
                name: "draining".to_string(),
                from: 21,
                to: 40,
                alert: Some(Alert {
                    severity: AlertSeverity::Info,
                    on_startup: true,
                    repeat_after_seconds: Some(60 * 20),
                    summary: "Battery is getting low. (${capacity}%)".to_string(),
                    message: None,
                    expire_after_seconds: Some(10),
                }),
            },
            BatteryPhase {
                name: "full".to_string(),
                from: 41,
                to: 99,
                alert: None,
            },
        ]
    }

    pub fn default_alert_battery_activated() -> Option<Alert> {
        Some(Alert {
            severity: AlertSeverity::Info,
            on_startup: true,
            repeat_after_seconds: None,
            summary: "Unplugged - switched to battery (${capacity}%)".to_string(),
            message: None,
            expire_after_seconds: None,
        })
    }

    pub fn default_alert_battery_deactivated() -> Option<Alert> {
        Some(Alert {
            severity: AlertSeverity::Info,
            on_startup: true,
            repeat_after_seconds: None,
            summary: "Plugged in! Battery is charging (${capacity}%)".to_string(),
            message: None,
            expire_after_seconds: Some(10),
        })
    }
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            refresh_interval_seconds: Self::default_refresh_interval_seconds(),
            phases: Self::default_phases(),
            alert_battery_activated: Self::default_alert_battery_activated(),
            alert_battery_deactivated: Self::default_alert_battery_deactivated(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BatteryPhase {
    pub name: String,
    pub from: u8,
    pub to: u8,
    pub alert: Option<Alert>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Range {
    Eq(u8),
    Gt(u8),
    Lt(u8),
}
