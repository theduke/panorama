use serde_derive::{Deserialize, Serialize};

use crate::cfg::{Alert, AlertSeverity};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FsConfig {
    pub enabled: bool,
    pub check_interval_secs: u64,
    pub disk_full_warning: Option<DiskUsageAlert>,
}

impl Default for FsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_interval_secs: 300,
            disk_full_warning: Some(DiskUsageAlert {
                usage_percent_limit: 95,
                device_path_exclude: None,
                fs_type_include: None,
                fs_type_exclude: None,
                alert: Alert {
                    severity: AlertSeverity::Warning,
                    on_startup: true,
                    repeat_after_seconds: None,
                    expire_after_seconds: Some(180),
                    summary: "Disk '{}' is almost full! (${usage_percent}%)".to_string(),
                    message: None,
                },
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DiskUsageAlert {
    pub usage_percent_limit: u8,
    pub device_path_exclude: Option<Vec<String>>,
    pub fs_type_include: Option<Vec<String>>,
    pub fs_type_exclude: Option<Vec<String>>,
    pub alert: Alert,
}
