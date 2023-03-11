use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde_derive::Deserialize;

use crate::error;
#[derive(Deserialize, Debug)]
pub struct Config {
    pub dns_seed: String,
    pub network_port: u32,
    pub start_string: String,
}

impl Config {
    pub fn new(config_file: String) -> error::Result<Self> {
        Ok(Figment::new()
            .merge(Toml::file(config_file))
            .merge(Env::prefixed("BTC_"))
            .extract::<Self>()?)
    }
}
