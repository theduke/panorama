use std::time::Duration;

// use dnsclient::UpstreamServer;
// use rand::seq::SliceRandom;

use anyhow::Context;

use crate::notify::Notifier;

use self::cfg::OnlineConfig;

pub mod cfg;

const ALERT_GROUP_INTERNET: &str = "panorama.internet";

pub struct OnlineManager {
    config: OnlineConfig,
    notifier: Notifier,
    offline_since: Option<std::time::SystemTime>,
    check_count: usize,
}

impl OnlineManager {
    pub async fn start(config: OnlineConfig, notifier: Notifier) -> Result<(), anyhow::Error> {
        let manager = Self::new(config, notifier)?;
        tokio::task::spawn_local(async move { manager.run().await })
            .await
            .context("OnlineManager taks failed")?
            .context("OnlineManager failed")?;

        Ok(())
    }

    fn new(config: OnlineConfig, notifier: Notifier) -> Result<Self, anyhow::Error> {
        let config = config.validate()?;

        Ok(Self {
            config,
            notifier,
            offline_since: None,
            check_count: 0,
        })
    }

    async fn run(mut self) -> Result<(), anyhow::Error> {
        loop {
            self.tick().await?;
            let time = if self.offline_since.is_some() {
                self.config.check_interval_seconds_offline
            } else {
                self.config.check_interval_seconds_online
            };
            tokio::time::sleep(std::time::Duration::from_secs(time)).await;
        }
    }

    async fn tick(&mut self) -> Result<(), anyhow::Error> {
        let mut count = 0;
        let is_online = 'OUTER: loop {
            // let servers = match &self.config.dns_servers {
            //     DnsServerSource::System => dnsclient::system::default_resolvers()?,
            //     DnsServerSource::Custom(servers) => {
            //         let mut servers = servers.clone();
            //         servers.shuffle(&mut rand::thread_rng());

            //         servers
            //             .iter()
            //             .map(|s| UpstreamServer::new((s.clone(), 53)))
            //             .collect()
            //     }
            // };

            // let client = dnsclient::sync::DNSClient::new(servers);

            // match client.query_a(&self.config.query_domain) {
            //     Ok(_addrs) => {
            //         break true;
            //     }
            //     Err(err) => {
            //         tracing::warn!(error=%err, "DNS online query failed");
            //         if count < self.config.retry_count {
            //             count += 1;
            //             std::thread::sleep(Duration::from_secs(self.config.retry_interval_seconds));
            //             continue;
            //         } else {
            //             break false;
            //         }
            //     }
            // }

            for check in &self.config.urls {
                let res = match ureq::get(check.url.as_str())
                    .timeout(Duration::from_secs(self.config.http_timeout_secs))
                    .call()
                    .context("http query failed")
                {
                    Ok(res) => {
                        if let Some(expected) = &check.body_contains {
                            match res.into_string().context("could not read response body") {
                                Ok(body) => {
                                    if body.contains(expected) {
                                        Ok(())
                                    } else {
                                        Err(anyhow::format_err!("response body did not contain expected string '{expected}'"))
                                    }
                                }
                                Err(err) => Err(err),
                            }
                        } else {
                            Ok(())
                        }
                    }
                    Err(err) => Err(err),
                };

                match res {
                    Ok(()) => {
                        break 'OUTER true;
                    }
                    Err(err) => {
                        tracing::warn!(url=%check.url, error=%err, "HTTP online query failed");
                        if count < self.config.retry_count {
                            count += 1;
                            tokio::time::sleep(Duration::from_secs(
                                self.config.retry_interval_seconds,
                            ))
                            .await;
                            continue;
                        } else {
                            break 'OUTER false;
                        }
                    }
                }
            }
        };

        if is_online {
            if self.offline_since.is_some() {
                self.offline_since = None;

                if let Some(alert) = &self.config.alert_reconnected {
                    let full = alert.prepare(ALERT_GROUP_INTERNET.to_string(), []);
                    self.notifier.notify(full).await?;
                }
            }
        } else if self.offline_since.is_none() {
            self.offline_since = Some(std::time::SystemTime::now());

            if let Some(alert) = &self.config.alert_disconnected {
                let full = alert.prepare(ALERT_GROUP_INTERNET.to_string(), []);
                self.notifier.notify(full).await?;
            }
        }

        self.check_count += 1;

        Ok(())
    }
}
