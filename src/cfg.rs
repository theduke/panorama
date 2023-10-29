use anyhow::Context;
use serde_derive::{Deserialize, Serialize};

use crate::{internet::cfg::OnlineConfig, power::cfg::PowerConfig};

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
    #[serde(default)]
    pub online: OnlineConfig,
}

impl Config {
    pub fn load_from_path(path: &std::path::Path) -> Result<Self, anyhow::Error> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file at '{}'", path.display()))?;

        let ext = path.extension().and_then(|x| x.to_str());
        let config = match ext {
            Some("yaml") => serde_yaml::from_str::<Self>(&content)
                .with_context(|| format!("Failed to parse config file at '{}'", path.display()))?,
            Some("toml") => toml::from_str::<Self>(&content)
                .with_context(|| format!("Failed to parse config file at '{}'", path.display()))?,
            _ => {
                return Err(anyhow::anyhow!(
                "Failed to parse config file at '{}': unknown extension - expected .toml or .yaml",
                path.display()
            ))
            }
        };
        Ok(config)
    }

    pub fn validate(mut self) -> Result<Self, anyhow::Error> {
        self.power = self.power.validate()?;
        Ok(self)
    }
}
