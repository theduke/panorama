use std::collections::HashMap;

use anyhow::Context;

use crate::cfg::{Alert, AlertSeverity};

#[derive(Debug)]
pub struct PreparedAlert {
    pub alert: Alert,
    pub group: Option<String>,
    pub variables: HashMap<String, String>,
}

#[derive(Clone)]
pub struct Notifier {
    sender: tokio::sync::mpsc::Sender<PreparedAlert>,
}

#[derive(Debug)]
struct State {
    receiver: tokio::sync::mpsc::Receiver<PreparedAlert>,
    category_ids: HashMap<String, u64>,
}

impl State {
    async fn run(&mut self) -> Result<(), anyhow::Error> {
        while let Some(msg) = self.receiver.recv().await {
            match self.notify(msg).await {
                Ok(()) => {}
                Err(err) => {
                    tracing::error!(error = &*err, "could not send inotify alert");
                }
            }
        }

        Ok(())
    }

    async fn notify(&mut self, alert: PreparedAlert) -> Result<(), anyhow::Error> {
        let urgency = match alert.alert.severity {
            AlertSeverity::Info => NotifyUrgency::Low,
            AlertSeverity::Warning => NotifyUrgency::Normal,
            AlertSeverity::Critical => NotifyUrgency::Critical,
        };

        let mut cmd = tokio::process::Command::new("notify-send");

        let mut summary = alert.alert.summary.clone();

        for (key, value) in &alert.variables {
            summary = summary.replace(&format!("${{{}}}", key), value);
        }
        let message = if let Some(msg) = &alert.alert.message {
            let mut msg = msg.clone();
            for (key, value) in &alert.variables {
                msg = msg.replace(&format!("${{{}}}", key), value);
            }
            Some(msg)
        } else {
            None
        };

        cmd
            // Print the notification ID so it can be replaced.
            .arg("--print-id")
            .args(["--app-name", "Panorama"])
            .args(["--urgency", urgency.as_str()]);

        if let Some(expire) = alert.alert.expire_after_seconds {
            cmd.arg(format!("--expire-time={}", expire * 1000));
        }

        if let Some(group) = &alert.group {
            let old_id = self.category_ids.get(group).copied();
            if let Some(old_id) = old_id {
                cmd.arg(format!("--replace-id={old_id}"));
            }
        }

        cmd.arg(summary);
        if let Some(message) = message {
            cmd.arg(message);
        }

        let out = cmd
            .output()
            .await
            .context("could not execute 'notify-send'")?;

        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let id = stdout
                .trim()
                .parse::<u64>()
                .context("could not parse notification ID")?;

            if let Some(group) = &alert.group {
                self.category_ids.insert(group.to_string(), id);
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

impl Notifier {
    pub fn start() -> (Self, tokio::task::JoinHandle<Result<(), anyhow::Error>>) {
        let (tx, rx) = tokio::sync::mpsc::channel(100);

        let state = State {
            receiver: rx,
            category_ids: HashMap::new(),
        };
        let join = tokio::task::spawn_local(async move {
            let mut state = state;
            let res = state.run().await;
            res
        });

        let s = Self { sender: tx };

        (s, join)
    }

    pub async fn notify(&self, alert: PreparedAlert) -> Result<(), anyhow::Error> {
        self.sender
            .send(alert)
            .await
            .map_err(|_| anyhow::anyhow!("alert channel was closed"))
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
