use bitcoin_p2p_example::{config::Config, error, protocol::handshake::start_handshake};
use clap::Parser;
use futures::future::join_all;
use log::{debug, info};
use simple_logger::SimpleLogger;
use tokio::{main, net::lookup_host};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Config file path, relative to the project root.
    #[arg(short, long, default_value_t = String::from("mainnet_config.toml"))]
    config_file: String,
}

#[main]
async fn main() {
    let result = run().await;
    if let Err(e) = result {
        log::error!("{:?}", e);
    }
}

async fn run() -> Result<(), error::Error> {
    // Parse cmd args
    let args = Args::parse();

    // Load config file with network parameters.
    let conf_result = Config::new(args.config_file);

    // Init logger
    let conf_result = match conf_result {
        Ok(conf) => {
            SimpleLogger::new()
                .env()
                .with_level(conf.get_log_level())
                .init()?;
            Ok(conf)
        }
        Err(e) => {
            SimpleLogger::new()
                .env()
                .with_level(log::LevelFilter::Info)
                .init()?;
            Err(e)
        }
    };
    let conf = conf_result?;
    conf.validate()?;

    info!("Loaded configuration: {:?}", conf);
    // Dns lookup
    info!("Start dns resolution");

    let dns_seed = format!("{}:{}", conf.dns_seed, conf.network_port);
    let iter = lookup_host(dns_seed).await?;
    let mut handles = Vec::new();
    iter.for_each(|elem| {
        debug!("Resolved socket {:?}", elem);
        let conf = conf.clone();
        let id = tokio::spawn(async move { start_handshake(elem, conf.start_string).await });
        handles.push(id);
    });
    let join_results = join_all(handles).await;
    for result in join_results{
        result?;
    }

    Ok(())
}
