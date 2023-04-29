// Loads config file
use serde_derive::Deserialize;
use std::fs;
use std::process::exit;
use toml;



#[derive(Deserialize)]
pub struct Config {
    pub roles: Roles,
}

//TODO: Try to convert toml definition to enum for cleaner abstraction. Or if not possible re-encapsulate into enum
#[derive(Deserialize)]
pub struct Roles {
    pub worker: bool,
    pub scheduler: bool
}

pub fn init_config() -> Config {
    let filename = "config.toml"; //TODO: Change to environment variable

    let contents = match fs::read_to_string(filename) {
        Ok(c) => c,
        Err(error) => {
            println!("{}",error);
            panic!("Could not read config file `{}`", filename);
        }
    };

    let config: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(error) => {
            println!("{}",error);
            panic!("Unable to load config data from `{}`", filename);
        }
    };

    return config;
}