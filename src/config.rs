// Loads config file
use serde_derive::Deserialize;
use std::fs;
use std::process::exit;
use toml;
use hashbrown::HashMap;

// let mut map = HashMap::new();
// map.insert(1, "one");

//TODO: Load config into global constant on load

#[derive(Deserialize)]
pub struct InternalConfig {
    pub workers: Vec<String>
}




#[derive(Deserialize)]
pub struct Config {
    pub roles: Roles,
    pub config: InternalConfig
}

pub struct ReconfiguredInternalConfig {
    pub workers: HashMap<u16,String>
}

pub struct ReconfiguredConfig {
    pub roles: Roles,
    pub config: ReconfiguredInternalConfig
}

//TODO: Try to convert toml definition to enum for cleaner abstraction. Or if not possible re-encapsulate into enum
#[derive(Deserialize)]
pub struct Roles {
    pub worker: bool,
    pub scheduler: bool
}

pub fn init_config() -> ReconfiguredConfig {
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

    let mut worker_hash = HashMap::new();

    for (i , worker) in config.config.workers.into_iter().enumerate() {
        worker_hash.insert(i as u16,worker);
    }

    let reconfigured_config : ReconfiguredConfig = ReconfiguredConfig {
        roles: config.roles,
        config: ReconfiguredInternalConfig {
            workers: worker_hash,
        },
    };


    return reconfigured_config;
}