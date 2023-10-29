pub mod cfg;
mod internet;
mod notify;
mod power;

use anyhow::Context;
use cfg::Config;
use power::PowerManager;

use crate::internet::OnlineManager;

pub type ResultCallback = Box<dyn Fn(Result<(), anyhow::Error>) + Send + Sync>;

pub struct App {
    #[allow(dead_code)]
    config: Config,
}

impl App {
    pub fn start(config: Config) -> Result<(), anyhow::Error> {
        let notifier = notify::Notifier::new();
        let (tx, rx) = std::sync::mpsc::channel();

        let mut has_checks = false;

        if config.power.enabled {
            has_checks = true;
            {
                let tx = tx.clone();
                let on_err = move |res: Result<(), anyhow::Error>| {
                    tx.send(res).unwrap();
                };

                PowerManager::start(config.power.clone(), Box::new(on_err), notifier.clone())?;
            }
        }

        if config.online.enabled {
            has_checks = true;
            {
                let tx = tx.clone();
                let on_err = move |res: Result<(), anyhow::Error>| {
                    tx.send(res).unwrap();
                };

                OnlineManager::start(config.online.clone(), Box::new(on_err), notifier.clone())?;
            }
        }

        if !has_checks {
            anyhow::bail!("No checks enabled - exiting");
        }

        tracing::info!("panorama has started");
        rx.recv().context("result channel died")?
    }
}
