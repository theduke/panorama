pub mod cfg;

use std::{collections::HashMap, time::Duration};

use anyhow::Context;

use crate::notify::Notifier;

use self::cfg::{DiskUsageAlert, FsConfig};

pub struct FsManager {
    config: FsConfig,
    /// Map recording whether a warning for a given disk was already sent.
    disk_full_warned: HashMap<String, bool>,
    notifier: Notifier,
}

impl FsManager {
    pub async fn start(config: FsConfig, notifier: Notifier) -> Result<(), anyhow::Error> {
        let mut manager = Self::new(config, notifier)?;
        tokio::task::spawn_local(async move { manager.run().await })
            .await
            .context("FsManager taks failed")?
            .context("FsManager failed")?;

        Ok(())
    }

    fn new(config: FsConfig, notifier: Notifier) -> Result<Self, anyhow::Error> {
        Ok(Self {
            config,
            disk_full_warned: HashMap::new(),
            notifier,
        })
    }

    async fn run(&mut self) -> Result<(), anyhow::Error> {
        let interval = Duration::from_secs(self.config.check_interval_secs);

        loop {
            tracing::trace!("loading mounts...");
            let mounts = tokio::task::spawn_blocking(load_proc_mounts)
                .await
                .context("load_mounts task failed")?
                .context("could not load active mounts")?;

            if let Some(full) = self.config.disk_full_warning.clone() {
                for mount in mounts {
                    self.handle_mount(mount, &full).await?;
                }
            }

            tokio::time::sleep(interval).await;
        }
    }

    async fn handle_mount(
        &mut self,
        mount: Mount,
        cfg: &DiskUsageAlert,
    ) -> Result<(), anyhow::Error> {
        if let Some(paths) = &cfg.device_path_exclude {
            if paths.contains(&mount.device) {
                return Ok(());
            }
        }

        if let Some(types) = &cfg.fs_type_exclude {
            if types.contains(&mount.fstype) {
                return Ok(());
            }
        }
        if let Some(types) = &cfg.fs_type_include {
            if !types.contains(&mount.fstype) {
                return Ok(());
            }
        }

        // FIXME: determine actual usage!
        let usage = 0;
        if usage < cfg.usage_percent_limit {
            return Ok(());
        }

        let already_warned = self
            .disk_full_warned
            .get(&mount.device)
            .copied()
            .unwrap_or_default();

        if already_warned {
            return Ok(());
        }

        let variables = {
            let mut vars = HashMap::new();
            vars.insert("usage_percent".to_string(), usage.to_string());
            vars.insert("device".to_string(), mount.device.clone());
            vars.insert("mountpoint".to_string(), mount.mountpoint.clone());
            vars.insert("fstype".to_string(), mount.fstype.clone());
            vars
        };
        let group = format!("fs-{}", mount.device);
        let alert = cfg.alert.prepare(group, variables);
        self.notifier.notify(alert).await?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Mount {
    device: String,
    mountpoint: String,
    fstype: String,
    options: Vec<String>,
}

fn load_proc_mounts() -> Result<Vec<Mount>, anyhow::Error> {
    let input =
        std::fs::read_to_string("/proc/mounts").with_context(|| "could not read /proc/mounts")?;

    parse_proc_mounts(&input)
}

fn parse_proc_mounts(input: &str) -> Result<Vec<Mount>, anyhow::Error> {
    input
        .trim()
        .lines()
        .map(|x| x.trim())
        .filter(|x| !x.is_empty())
        .map(parse_proc_mount_line)
        .collect()
}

fn parse_proc_mount_line(line: &str) -> Result<Mount, anyhow::Error> {
    let mut parts = line.split_whitespace();

    let device = parts
        .next()
        .with_context(|| format!("could not read device from line '{line}'"))?
        .to_string();

    let mountpoint = parts
        .next()
        .with_context(|| format!("could not read mountpoint from line '{line}'"))?
        .to_string();

    let fstype = parts
        .next()
        .with_context(|| format!("could not read fstype from line '{line}'"))?
        .to_string();

    let options = parts
        .next()
        .with_context(|| format!("could not read options from line '{line}'"))?
        .split(',')
        .map(|x| x.to_string())
        .collect();

    Ok(Mount {
        device,
        mountpoint,
        fstype,
        options,
    })
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_parse_proc_mounts() {
        let input = r#"
devtmpfs /dev devtmpfs rw,nosuid,size=1604,nr_inodes=407,mode=755 0 0
devpts /dev/pts devpts rw,nosuid,noexec,relatime,gid=3,mode=620,ptmxmode=666 0 0
tmpfs /dev/shm tmpfs rw,nosuid,nodev 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
tmpfs /run tmpfs rw,nosuid,nodev,size=8123,mode=755 0 0
ramfs /run/keys ramfs rw,nosuid,nodev,relatime,mode=750 0 0
        "#;

        let mounts = parse_proc_mounts(input).unwrap();

        assert_eq!(
            mounts,
            vec![
                Mount {
                    device: "devtmpfs".to_string(),
                    mountpoint: "/dev".to_string(),
                    fstype: "devtmpfs".to_string(),
                    options: vec![
                        "rw".to_string(),
                        "nosuid".to_string(),
                        "size=1604".to_string(),
                        "nr_inodes=407".to_string(),
                        "mode=755".to_string(),
                    ]
                },
                Mount {
                    device: "devpts".to_string(),
                    mountpoint: "/dev/pts".to_string(),
                    fstype: "devpts".to_string(),
                    options: vec![
                        "rw".to_string(),
                        "nosuid".to_string(),
                        "noexec".to_string(),
                        "relatime".to_string(),
                        "gid=3".to_string(),
                        "mode=620".to_string(),
                        "ptmxmode=666".to_string(),
                    ]
                },
                Mount {
                    device: "tmpfs".to_string(),
                    mountpoint: "/dev/shm".to_string(),
                    fstype: "tmpfs".to_string(),
                    options: vec!["rw".to_string(), "nosuid".to_string(), "nodev".to_string(),]
                },
                Mount {
                    device: "proc".to_string(),
                    mountpoint: "/proc".to_string(),
                    fstype: "proc".to_string(),
                    options: vec![
                        "rw".to_string(),
                        "nosuid".to_string(),
                        "nodev".to_string(),
                        "noexec".to_string(),
                        "relatime".to_string()
                    ]
                },
                Mount {
                    device: "tmpfs".to_string(),
                    mountpoint: "/run".to_string(),
                    fstype: "tmpfs".to_string(),
                    options: vec![
                        "rw".to_string(),
                        "nosuid".to_string(),
                        "nodev".to_string(),
                        "size=8123".to_string(),
                        "mode=755".to_string()
                    ]
                },
                Mount {
                    device: "ramfs".to_string(),
                    mountpoint: "/run/keys".to_string(),
                    fstype: "ramfs".to_string(),
                    options: vec![
                        "rw".to_string(),
                        "nosuid".to_string(),
                        "nodev".to_string(),
                        "relatime".to_string(),
                        "mode=750".to_string()
                    ]
                },
            ]
        );
    }
}
