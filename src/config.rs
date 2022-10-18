use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, File};
use std::io::Write;
use tracing::warn;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::load_config);

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub db: String,
    pub min_id: Option<u32>,
    pub max_id: u32,
    pub book_cover_path: String,
}

impl Config {
    fn load_config() -> Config {
        let cfg_file = std::env::args()
            .nth(1)
            .unwrap_or_else(|| "config.toml".to_owned());
        if let Ok(config_toml_content) = read_to_string(cfg_file) {
            let config: Config = toml::from_str(&config_toml_content).unwrap();
            config
        } else {
            warn!("Config file not found, using default config.toml");
            let config = Config::default();
            let toml = toml::to_string_pretty(&config).unwrap();
            let mut cfg_file = File::create("config.toml").unwrap();
            cfg_file.write_all(toml.as_bytes()).unwrap();
            config
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            db: "extract.db".into(),
            min_id: None,
            max_id: 512631,
            book_cover_path: "books".into(),
        }
    }
}
