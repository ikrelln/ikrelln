use std::fs::File;
use std::io::prelude::*;

use structopt::StructOpt;
use toml;

#[derive(Debug, Clone)]
pub struct CleanUpConfig {
    pub delay_test_results: u32,
    pub delay_spans: u32,
    pub delay_reports: u32,
    pub schedule: u32,
}
impl Default for CleanUpConfig {
    fn default() -> Self {
        CleanUpConfig {
            delay_test_results: 40 * 24 * 60 * 60 * 1000,
            delay_spans: 7 * 24 * 60 * 60 * 1000,
            delay_reports: 14 * 24 * 60 * 60 * 1000,
            schedule: 60 * 60 * 1000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub db_url: String,
    pub cleanup: CleanUpConfig,
}
impl Default for Config {
    fn default() -> Self {
        Config {
            host: "0.0.0.0".to_string(),
            port: 7878,
            db_url: "127.0.0.1:5042".to_string(),
            cleanup: CleanUpConfig::default(),
        }
    }
}

impl Config {
    pub fn load() -> Config {
        let config = merge_configs();
        match config {
            Ok(config) => config,
            Err(err) => panic!("{:?}", err),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename = "cleanup")]
pub struct CleanUpConfigLoader {
    pub delay_test_results: Option<u32>,
    pub delay_spans: Option<u32>,
    pub delay_reports: Option<u32>,
    pub schedule: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ConfigLoader {
    pub host: Option<String>,
    pub port: Option<u16>,
    pub db_nb_connection: Option<usize>,
    pub db_url: Option<String>,
    pub cleanup: Option<CleanUpConfigLoader>,
}

#[derive(Debug, Clone, Deserialize, StructOpt)]
pub struct ConfigLoaderCmd {
    #[structopt(
        short = "h",
        long = "host",
        env = "HOST",
        help = "Listen on the specified host, by default 0.0.0.0"
    )]
    pub host: Option<String>,
    #[structopt(
        short = "p",
        long = "port",
        env = "PORT",
        help = "Listen on the specified host, by default 7878"
    )]
    pub port: Option<u16>,
    #[structopt(
        long = "db-url",
        env = "DATABASE_URL",
        help = "URL to connect to the database"
    )]
    pub db_url: Option<String>,
}

fn load_config_from_toml() -> ConfigLoader {
    let contents = File::open("config.toml").and_then(|mut file| {
        let mut contents = String::new();
        file.read_to_string(&mut contents).map(|_| contents)
    });
    let config: Option<ConfigLoader> = contents
        .ok()
        .and_then(|contents| toml::from_str(&contents).ok());

    config.unwrap_or_else(ConfigLoader::default)
}

fn merge_configs() -> Result<Config, String> {
    let from_args = ConfigLoaderCmd::from_args();
    let from_toml = load_config_from_toml();
    let cleanup_from_toml = from_toml.cleanup;
    let default = Config::default();

    Ok(Config {
        port: from_args.port.or(from_toml.port).unwrap_or(default.port),
        host: from_args.host.or(from_toml.host).unwrap_or(default.host),
        db_url: from_args
            .db_url
            .or(from_toml.db_url)
            .ok_or("missing DATABASE_URL parameter")?,
        cleanup: CleanUpConfig {
            delay_test_results: cleanup_from_toml
                .clone()
                .and_then(|cleanup| cleanup.delay_test_results)
                .unwrap_or(default.cleanup.delay_test_results),
            delay_spans: cleanup_from_toml
                .clone()
                .and_then(|cleanup| cleanup.delay_spans)
                .unwrap_or(default.cleanup.delay_spans),
            delay_reports: cleanup_from_toml
                .clone()
                .and_then(|cleanup| cleanup.delay_reports)
                .unwrap_or(default.cleanup.delay_reports),
            schedule: cleanup_from_toml
                .clone()
                .and_then(|cleanup| cleanup.schedule)
                .unwrap_or(default.cleanup.schedule),
        },
    })
}
