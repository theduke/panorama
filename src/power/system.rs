use std::path::Path;

use anyhow::Context;

#[derive(Clone, Debug)]
pub struct PowerSupply {
    pub name: String,
    pub kind: PowerSupplyType,
}

#[derive(Clone, Debug)]
pub enum PowerSupplyType {
    Battery(PowerSupplyBattery),
    Main(PowerSupplyMain),
}

impl PowerSupplyType {
    pub fn as_main(&self) -> Option<&PowerSupplyMain> {
        if let Self::Main(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct PowerSupplyBattery {
    pub status: BatteryStatus,
    pub capacity: u8,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BatteryStatus {
    Unknown,
    Charging,
    Discharging,
    NotCharging,
    Full,
}

#[derive(Clone, Debug)]
pub struct PowerSupplyMain {
    pub online: bool,
}

pub fn read_all_supplies() -> Result<Vec<PowerSupply>, anyhow::Error> {
    std::fs::read_dir("/sys/class/power_supply")?
        .map(|res| {
            let entry = res.context("could not read power supply entry")?;
            let supply = read_supply(&entry.path())?;
            Ok(supply)
        })
        .collect()
}

pub fn read_supply(path: &Path) -> Result<PowerSupply, anyhow::Error> {
    let name = path
        .file_name()
        .context("could not read name from file path")?
        .to_str()
        .context("could not convert name to string")?
        .to_string();

    let kind_name =
        std::fs::read_to_string(path.join("type")).context("could not read power supply type")?;
    let kind = match kind_name.trim() {
        "Battery" => {
            let status_raw = std::fs::read_to_string(path.join("status"))
                .context("could not read battery status")?;

            let status = match status_raw.trim() {
                "Unknown" => BatteryStatus::Unknown,
                "Charging" => BatteryStatus::Charging,
                "Discharging" => BatteryStatus::Discharging,
                "Not charging" => BatteryStatus::NotCharging,
                "Full" => BatteryStatus::Full,
                other => {
                    anyhow::bail!("unknown battery status: {}", other)
                }
            };

            let capacity = std::fs::read_to_string(path.join("capacity"))
                .context("could not read battery capacity")?
                .trim()
                .parse::<u8>()
                .context("could not parse battery capacity")?;

            PowerSupplyType::Battery(PowerSupplyBattery { status, capacity })
        }
        "Mains" => {
            let online_raw = std::fs::read_to_string(path.join("online"))
                .context("could not read main power supply status")?
                .trim()
                .parse::<u8>()
                .context("could not parse main power supply status")?;
            let online = online_raw == 1;
            PowerSupplyType::Main(PowerSupplyMain { online })
        }
        other => {
            anyhow::bail!("unknown power supply type: '{}'", other)
        }
    };

    Ok(PowerSupply { name, kind })
}
