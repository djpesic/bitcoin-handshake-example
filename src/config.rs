use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde_derive::Deserialize;

use crate::error;
#[derive(Deserialize, Debug)]
pub struct Config {
    dns_seed: String,
    network_port: u32,
    start_string: String,
    max_nbits: String,
}

impl Config {
    pub fn new(config_file: String) -> error::Result<Self> {
        Ok(Figment::new()
            .merge(Toml::file(config_file))
            .merge(Env::prefixed("BTC_"))
            .extract::<Self>()?)
    }
}
