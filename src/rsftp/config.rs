use serde::Deserialize;
use std::fs;
use toml;

#[derive(Deserialize, Debug)]
pub struct SyncPath {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub private_key_path: String,
    pub local_path: String,
    pub remote_path: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub paths: Vec<SyncPath>,
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string("/Users/dizzrt/Desktop/rust/rsftp/tests/config.toml")?;
    let config: Config = toml::from_str(&config_content)?;

    Ok(config)
}
