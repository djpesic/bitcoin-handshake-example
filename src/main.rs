use bitcoin_p2p_example::{config::Config, error};
use clap::Parser;
use log::info;
use simple_logger::SimpleLogger;
use tokio::{main, net::lookup_host};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Config file path, relative to the project root.
    #[arg(short, long, default_value_t = String::from("mainnet_config.toml"))]
    config_file: String
}

#[main]
async fn main() {
    let result = run().await;
    if let Err(e) = result{
        log::error!("{:?}",e);
    }
}

async fn run()->Result<(), error::Error>{
    SimpleLogger::new().init()?;

    // Parse cmd args
    let args = Args::parse();

    // Load config file with network parameters.
    let conf = Config::new(args.config_file)?;
    info!("Loaded config: {:?}", conf);

    // Dns lookup
    let dns_seed = format!("{}:{}", conf.dns_seed, conf.network_port);
    let iter = lookup_host(dns_seed).await?;
    iter.for_each(|elem|{
        info!("Got socket {:?}", elem);
    });
    Ok(())
}