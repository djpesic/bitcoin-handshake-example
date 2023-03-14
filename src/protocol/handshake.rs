use std::net::{IpAddr, SocketAddr};

use log::{debug, info};
use rand::prelude::*;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::{
    error,
    protocol::messages::{verack::Verack, version::Version, Message},
};

pub async fn start_handshake(
    socket: std::net::SocketAddr,
    start_string: String,
) -> error::Result<()> {
    let result = run_handshake(socket, start_string).await;
    if result.is_err() {
        log::error!(target:"handshake", "{:?}", result);
    }
    result
}

fn generate_nonce() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen::<u64>()
}

// If socket has ipv4, convert to ipv6 mapped.
fn to_ipv6(sock: &mut SocketAddr) {
    let mapped = match sock.ip() {
        IpAddr::V4(ip) => ip.to_ipv6_mapped(),
        IpAddr::V6(ip) => ip,
    };

    sock.set_ip(IpAddr::V6(mapped));
}
async fn run_handshake(
    socket: std::net::SocketAddr,
    start_string: String,
) -> Result<(), error::Error> {
    let mut socket = socket;
    to_ipv6(&mut socket);

    info!(target:"handshake", "{:?} Starting handshake", socket);
    let mut socket = socket;
    // Connect to a remote node
    let mut stream = TcpStream::connect(socket).await?;

    handshake_version(&mut stream, &mut socket, start_string.clone()).await?;

    handshake_verack(&mut stream, &mut socket, start_string).await?;

    Ok(())
}

async fn handshake_verack(
    stream: &mut TcpStream,
    socket: &mut SocketAddr,
    start_string: String,
) -> Result<(), error::Error> {
    // Send verack message
    let mut msg = Verack::new(start_string);
    info!(target:"handshake", "{:?} Sending verack message", socket);
    debug!(target:"handshake", "{:?} Message data: {:#x?}", socket, msg);
    let serialized_msg = msg.to_bytes()?;
    stream.write_all(&serialized_msg).await?;

    // Receive verack
    let reader = BufReader::new(serialized_msg.as_slice());
    let msg1 = Verack::from_bytes(reader).await?;
    info!(target:"handshake", "{:?} Receive verack message", socket);
    debug!(target:"handshake", "{:?} Message data: {:#x?}",socket, msg1);
    Ok(())
}

async fn handshake_version(
    stream: &mut TcpStream,
    socket: &mut SocketAddr,
    start_string: String,
) -> Result<(), error::Error> {
    let mut recv = stream.local_addr()?;

    to_ipv6(&mut recv);

    // Send version message
    let mut send_version = Version::new(start_string.clone(), generate_nonce(), *socket, recv)?;
    info!(target:"handshake", "{:?} Sending version message", socket);
    let serialized_msg = send_version.to_bytes()?;
    debug!(target:"handshake", "{:?} Message data: {:#x?}", socket, send_version);
    stream.write_all(&serialized_msg).await?;

    // Receive version
    let reader = BufReader::new(stream);

    let received_version = Version::from_bytes(reader).await?;
    info!(target:"handshake", "{:?} Received version messasge", socket);
    debug!(target:"handshake", "{:?} Message data: {:#x?}", socket, received_version);

    if send_version.get_nonce() == received_version.get_nonce() {
        return Err(error::Error::NonceConflictError);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::protocol::messages::verack::Verack;
    use crate::protocol::messages::Message;
    use tokio::io::BufReader;

    #[tokio::test]
    async fn test_serde() {
        let mut msg = Verack::new(String::from("f9beb4d9"));
        let serialized_msg = msg.to_bytes().unwrap();
        let reader = BufReader::new(serialized_msg.as_slice());
        let msg1 = Verack::from_bytes(reader).await.unwrap();
        let str1 = format!("{:?}", msg);
        let str2 = format!("{:?}", msg1);
        assert_eq!(str1, str2);
    }
}
