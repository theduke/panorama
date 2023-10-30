pub mod cfg;
mod fs;
mod internet;
mod notify;
mod power;
mod udev;

use anyhow::Context;
use cfg::Config;
use futures::{future::LocalBoxFuture, stream::FuturesUnordered, StreamExt};
use power::PowerManager;

use crate::internet::OnlineManager;

pub type ResultCallback = Box<dyn Fn(Result<(), anyhow::Error>) + Send + Sync>;

pub struct App {
    #[allow(dead_code)]
    config: Config,
}

impl App {
    pub async fn run(config: Config) -> Result<(), anyhow::Error> {
        // Everything runs in a single-threaded tokio runtime, so a LocalSet can
        // be used to avoid send/sync issues.
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move { Self::run_inner(config).await })
            .await
    }

    pub async fn run_inner(config: Config) -> Result<(), anyhow::Error> {
        let (notifier, notifier_join) = notify::Notifier::start();

        let mut tasks =
            FuturesUnordered::<LocalBoxFuture<'static, Result<(), anyhow::Error>>>::new();

        if config.power.enabled {
            let fut = PowerManager::start(config.power.clone(), notifier.clone());
            tasks.push(Box::pin(fut));
        }
        if config.online.enabled {
            let fut = OnlineManager::start(config.online.clone(), notifier.clone());
            tasks.push(Box::pin(fut));
        }
        if config.fs.enabled {
            let fut = fs::FsManager::start(config.fs.clone(), notifier.clone());
            tasks.push(Box::pin(fut));
        }

        let fut = tokio::task::spawn_local(async move { udev::run().await });
        let fut = async move { fut.await.context("udev task failed")? };
        tasks.push(Box::pin(fut));

        if tasks.is_empty() {
            anyhow::bail!("No checks enabled - exiting");
        }
        let notifier_fut = async move {
            notifier_join.await.context("notifier failed")??;
            Ok(())
        };
        tasks.push(Box::pin(notifier_fut));

        tracing::info!("panorama has started");

        tasks.next().await.context("task failed")??;

        Ok(())
    }
}
