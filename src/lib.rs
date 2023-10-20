use anyhow::Context;
use cfg::Config;
use power::PowerManager;

pub mod cfg;
mod notify;
mod power;

pub type ResultCallback = Box<dyn Fn(Result<(), anyhow::Error>) + Send + Sync>;

pub struct App {
    #[allow(dead_code)]
    config: Config,
}

impl App {
    pub fn start(config: Config) -> Result<(), anyhow::Error> {
        let notifier = notify::Notifier::new();
        let (tx, rx) = std::sync::mpsc::channel();

        {
            let tx = tx.clone();
            let on_err = move |res: Result<(), anyhow::Error>| {
                tx.send(res).unwrap();
            };

            PowerManager::start(config.power.clone(), Box::new(on_err), notifier.clone())?;
        }

        tracing::info!("panorama has started");

        rx.recv().context("result channel died")?
    }
}
