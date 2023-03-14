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
        log::error!(target:"", "{:?}", e);
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
                .with_colors(true)
                .init()?;
            Ok(conf)
        }
        Err(e) => {
            SimpleLogger::new()
                .env()
                .with_level(log::LevelFilter::Info)
                .with_colors(true)
                .init()?;
            Err(e)
        }
    };
    let conf = conf_result?;
    conf.validate()?;

    info!(target:"", "Loaded configuration: {:?}", conf);
    // Dns lookup
    info!(target:"", "Start dns resolution");

    let dns_seed = format!("{}:{}", conf.dns_seed, conf.network_port);
    let iter = lookup_host(dns_seed).await?;
    let mut handles = Vec::new();
    iter.for_each(|elem| {
        debug!(target:"", "Resolved socket {:?}", elem);
        let conf = conf.clone();
        let id = tokio::spawn(async move { start_handshake(elem, conf.start_string).await });
        handles.push(id);
    });
    let join_results = join_all(handles).await;
    let mut num_ok = 0;
    let len = join_results.len();
    join_results.into_iter().flatten().for_each(|elem| {
        if elem.is_ok() {
            num_ok += 1;
        }
    });
    info!(target:"", "Handshaking finished.");
    info!(target:"", "Number of successfull handshakes: {}, number of failed handshakes {}", num_ok, len-num_ok);
    Ok(())
}
