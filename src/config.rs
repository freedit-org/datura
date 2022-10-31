use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;

pub static CONFIG: Lazy<Config> = Lazy::new(Config::load_config);

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub db: String,
    pub book_cover_path: String,
    pub movie_cover_path: String,
    pub album_cover_path: String,
}

impl Config {
    fn load_config() -> Config {
        let cfg_file = std::env::args()
            .nth(1)
            .unwrap_or_else(|| "config.toml".to_owned());
        let config_toml_content = read_to_string(cfg_file).unwrap();
        let config: Config = toml::from_str(&config_toml_content).unwrap();
        config
    }
}
