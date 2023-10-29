use std::net::IpAddr;

use serde_derive::{Deserialize, Serialize};

use crate::cfg::{Alert, AlertSeverity};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OnlineConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "OnlineConfig::default_dns_servers")]
    pub dns_servers: DnsServerSource,

    /// HTTP URLs to check for internet connectivity.
    #[serde(default = "OnlineConfig::default_urls")]
    pub urls: Vec<CheckUrl>,

    #[serde(default = "OnlineConfig::default_http_timeout_secs")]
    pub http_timeout_secs: u64,

    #[serde(default = "OnlineConfig::default_query_domain")]
    pub query_domain: String,

    #[serde(default = "OnlineConfig::default_check_interval_seconds_online")]
    pub check_interval_seconds_online: u64,

    #[serde(default = "OnlineConfig::default_check_interval_seconds_offline")]
    pub check_interval_seconds_offline: u64,

    /// How often to retry failed checks before system is considered offline.
    /// Also check retry_delay_seconds.
    #[serde(default = "OnlineConfig::default_retry_count")]
    pub retry_count: usize,
    #[serde(default = "OnlineConfig::default_retry_interval_seconds")]
    pub retry_interval_seconds: u64,

    pub alert_reconnected: Option<Alert>,
    pub alert_disconnected: Option<Alert>,
}

impl OnlineConfig {
    pub fn validate(self) -> Result<Self, anyhow::Error> {
        match &self.dns_servers {
            DnsServerSource::System => {}
            DnsServerSource::Custom(servers) => {
                if servers.is_empty() {
                    anyhow::bail!("'online.dns_servers' must specify at least one server");
                }
            }
        };
        Ok(self)
    }

    fn default_http_timeout_secs() -> u64 {
        20
    }

    fn default_urls() -> Vec<CheckUrl> {
        vec![
            CheckUrl {
                url: "https://wikipedia.org".parse().unwrap(),
                body_contains: Some("Wikimedia Foundation".to_string()),
            },
            CheckUrl {
                url: "https://news.ycombinator.com".parse().unwrap(),
                body_contains: Some("Hacker News".to_string()),
            },
        ]
    }

    fn default_dns_servers() -> DnsServerSource {
        DnsServerSource::Custom(vec!["1.1.1.1".parse().unwrap(), "8.8.8.8".parse().unwrap()])
    }

    fn default_query_domain() -> String {
        "google.com".to_string()
    }

    fn default_check_interval_seconds_online() -> u64 {
        30
    }

    fn default_check_interval_seconds_offline() -> u64 {
        3
    }

    fn default_retry_count() -> usize {
        2
    }

    fn default_retry_interval_seconds() -> u64 {
        5
    }
}

impl Default for OnlineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            http_timeout_secs: Self::default_http_timeout_secs(),
            query_domain: Self::default_query_domain(),
            dns_servers: Self::default_dns_servers(),
            check_interval_seconds_online: Self::default_check_interval_seconds_online(),
            check_interval_seconds_offline: Self::default_check_interval_seconds_offline(),
            retry_count: Self::default_retry_count(),
            retry_interval_seconds: Self::default_retry_interval_seconds(),
            alert_reconnected: Some(Alert {
                severity: AlertSeverity::Info,
                on_startup: false,
                repeat_after_seconds: None,
                expire_after_seconds: Some(10),
                summary: "Internet is reachable!".to_string(),
                message: None,
            }),
            alert_disconnected: Some(Alert {
                severity: AlertSeverity::Critical,
                on_startup: false,
                repeat_after_seconds: None,
                expire_after_seconds: None,
                summary: "Internet is unreachable - system appears to be offline!".to_string(),
                message: None,
            }),
            urls: Self::default_urls(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CheckUrl {
    pub url: url::Url,
    pub body_contains: Option<String>,
}

/// DNS servers to use for online checks.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DnsServerSource {
    System,
    Custom(Vec<IpAddr>),
}

fn default_true() -> bool {
    true
}
