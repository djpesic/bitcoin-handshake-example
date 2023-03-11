use bitcoin_p2p_example::config::Config;

fn main() {
    let conf = Config::new(String::from("mainnet_config.toml"));
    println!("Config {:?}", conf);
}
