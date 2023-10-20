use anyhow::Context;
use serde_derive::{Deserialize, Serialize};

use crate::power::cfg::PowerConfig;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Alert {
    pub severity: AlertSeverity,
    pub on_startup: bool,
    pub repeat_after_seconds: Option<u64>,
    pub expire_after_seconds: Option<u64>,
    pub summary: String,
    pub message: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Config {
    #[serde(default)]
    pub power: PowerConfig,
}

impl Config {
    pub fn load_from_path(path: &std::path::Path) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file at '{}'", path.display()))?;
        let config = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file at '{}'", path.display()))?;
        Ok(config)
    }

    pub fn validate(mut self) -> Result<Self, anyhow::Error> {
        self.power = self.power.validate()?;
        Ok(self)
    }
}
