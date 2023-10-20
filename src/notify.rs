use std::{
    collections::HashMap,
    process::Command,
    sync::{Arc, Mutex},
};

use anyhow::Context;

use crate::cfg::{Alert, AlertSeverity};

#[derive(Clone, Debug)]
pub struct Notifier {
    state: Arc<Mutex<State>>,
}

#[derive(Debug)]
struct State {
    category_ids: HashMap<String, u64>,
}

impl Notifier {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                category_ids: HashMap::new(),
            })),
        }
    }

    pub fn notify(&self, alert: &Alert, group: Option<&str>) -> Result<(), anyhow::Error> {
        let urgency = match alert.severity {
            AlertSeverity::Info => NotifyUrgency::Low,
            AlertSeverity::Warning => NotifyUrgency::Normal,
            AlertSeverity::Critical => NotifyUrgency::Critical,
        };

        let mut cmd = Command::new("notify-send");
        cmd.arg(&alert.summary)
            // Print the notification ID so it can be replaced.
            .arg("--print-id")
            .args(["--app-name", "Panorama"])
            .args(["--urgency", urgency.as_str()]);

        if let Some(expire) = alert.expire_after_seconds {
            cmd.arg(format!("--expire-time={}", expire * 1000));
        }

        if let Some(group) = group {
            let old_id = self.state.lock().unwrap().category_ids.get(group).copied();
            if let Some(old_id) = old_id {
                cmd.arg(format!("--replace-id={old_id}"));
            }
        }

        let out = cmd.output().context("could not execute 'notify-send'")?;

        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let id = stdout
                .trim()
                .parse::<u64>()
                .context("could not parse notification ID")?;

            if let Some(group) = group {
                let mut state = self.state.lock().unwrap();
                state.category_ids.insert(group.to_string(), id);
            }

            Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            Err(anyhow::anyhow!(
                "'notify-send' exited with non-zero status: {stdout} {stderr}"
            ))
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum NotifyUrgency {
    Low,
    Normal,
    Critical,
}

impl NotifyUrgency {
    fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Normal => "normal",
            Self::Critical => "critical",
        }
    }
}
