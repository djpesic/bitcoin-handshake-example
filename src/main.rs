use bitcoin_p2p_example::config::Config;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// TCP port to connect to.
    #[arg(short, long, default_value_t = String::from("mainnet_config.toml"))]
    config_file: String
}

fn main() {
    let args = Args::parse();

    let conf = Config::new(args.config_file);
    println!("Config {:?}", conf);
}
