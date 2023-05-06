use std::fs;
use log::error;
// Loads config file
use serde_derive::Deserialize;


//TODO: Load config into global constant on load

#[derive(Deserialize,Clone)]
pub struct InternalConfig {
    pub discord_bot_id: u64,
    pub discord_bot_token: String,
}

#[derive(Deserialize,Clone)]
pub struct Config {
    pub roles: Roles,
    pub config: InternalConfig
}

#[derive(Deserialize,Clone)]
pub struct Roles {
    pub worker: bool,
    pub scheduler: bool
}

pub fn init_config() -> Config {
    let filename = "config.toml"; //TODO: Change to environment variable

    let contents = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(error) => {
            error!("{}",error);
            panic!("Could not read config file `{}`", filename);
        }
    };

    let config: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(error) => {
            error!("{}",error);
            panic!("Unable to load config data from `{}`", filename);
        }
    };

    return config;
}