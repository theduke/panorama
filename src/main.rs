use std::path::PathBuf;

use anyhow::Context;
use panoramas::{cfg::Config, App};

struct Cli {
    config_path: Option<String>,
    verbose: bool,
    help: bool,
    dump_default_config: bool,
}

impl Cli {
    const USAGE: &'static str = r#"panorama - a status daemon for Linux

USAGE:

* -c/--config <path> - path to the config file
* -v/--verbose - enable verbose logging
* -h/--help - show this help message
* --dump-default-config - show the default config file and exit
"#;

    fn parse_env() -> Result<Self, anyhow::Error> {
        let args = std::env::args();
        Self::parse(args)
    }

    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self, anyhow::Error> {
        let _cmd_name = args.next().unwrap();

        let mut s = Cli {
            config_path: None,
            verbose: false,
            help: false,
            dump_default_config: false,
        };

        while let Some(val) = args.next() {
            match val.as_str() {
                "-c" | "--config" => {
                    let path = args.next().context("-c/--config requires a path")?;
                    s.config_path = Some(path);
                }
                "-v" | "--verbose" => {
                    s.verbose = true;
                }
                "-h" | "--help" => {
                    s.help = true;
                }
                "--dump-default-config" => {
                    s.dump_default_config = true;
                }
                other => {
                    anyhow::bail!("unknown argument '{other}'");
                }
            }
        }

        Ok(s)
    }

    fn run(self) -> Result<(), anyhow::Error> {
        if self.help {
            eprintln!("{}", Self::USAGE);
            return Ok(());
        }
        if self.dump_default_config {
            let config = Config::default();
            let content = serde_yaml::to_string(&config)?;
            println!("# Default config for panorama\n\n{content}");
            return Ok(());
        }

        if std::env::var("RUST_LOG").is_err() {
            let filter = if self.verbose { "trace" } else { "info" };
            std::env::set_var("RUST_LOG", filter);
        }
        tracing_subscriber::fmt::init();

        let config_path = if let Some(path) = self.config_path {
            let p = PathBuf::from(path);
            if !p.is_file() {
                anyhow::bail!("config path '{}' is not a file", p.display());
            }
            Some(p)
        } else {
            #[allow(deprecated)]
            let default_path = std::env::home_dir()
                .expect("failed to determine home dir")
                .join(".config")
                .join("panorama")
                .join("config");

            let toml = default_path.with_extension("toml");

            if toml.is_file() {
                Some(default_path)
            } else {
                let yaml = default_path.with_extension("yaml");
                if yaml.is_file() {
                    Some(default_path)
                } else {
                    None
                }
            }
        };

        let config = if let Some(path) = &config_path {
            tracing::info!("loading config from '{}'", path.display());
            Config::load_from_path(path)?
        } else {
            tracing::info!("no config found or specified - using built-in default config");
            Config::default()
        };

        App::start(config)
    }
}

fn main() {
    let cli = Cli::parse_env().unwrap();
    cli.run().unwrap();
}
