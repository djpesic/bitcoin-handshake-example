use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde_derive::Deserialize;

use crate::error;
#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub dns_seed: String,
    pub network_port: u32,
    pub start_string: String,
    log_level: String,
}
impl Config {
    pub fn get_log_level(&self) -> log::LevelFilter {
        if self.log_level == "debug" {
            log::LevelFilter::Debug
        } else if self.log_level == "error" {
            log::LevelFilter::Error
        } else if self.log_level == "trace" {
            log::LevelFilter::Trace
        } else if self.log_level == "warn" {
            log::LevelFilter::Warn
        } else if self.log_level == "off" {
            log::LevelFilter::Off
        } else {
            log::LevelFilter::Info
        }
    }

    pub fn validate(&self) -> Result<(), error::Error> {
        let log_levels: Vec<&str> = vec!["debug", "error", "trace", "warn", "off", "info"];

        if !log_levels.contains(&self.log_level.as_str()) {
            return Err(error::Error::ConfigDataEror(String::from(
                "Invalid log level.",
            )));
        }
        if (self.network_port != 8333) && (self.network_port != 18333) {
            return Err(error::Error::ConfigDataEror(String::from(
                "Invalid network port.",
            )));
        }
        if (self.start_string != "0b110907") && (self.start_string != "f9beb4d9") {
            return Err(error::Error::ConfigDataEror(String::from(
                "Invalid start string",
            )));
        }
        Ok(())
    }
}

impl Config {
    pub fn new(config_file: String) -> error::Result<Self> {
        let config = Figment::new()
            .merge(Toml::file(config_file))
            .merge(Env::prefixed("BTC_"))
            .extract::<Self>()?;
        Ok(config)
    }
}
