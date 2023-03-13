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

pub async fn start_handshake(socket: std::net::SocketAddr, start_string: String) {
    let result = run_handshake(socket, start_string).await;
    if let Err(e) = result {
        log::error!("{:?}", e);
    }
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
    info!("Starting handshake with {:?}", socket);
    let mut socket = socket;
    // Connect to a remote node
    let mut stream = TcpStream::connect(socket).await?;

    handshake_version(&mut stream, &mut socket, start_string.clone()).await?;

    handshake_verack(&mut stream, start_string).await?;

    Ok(())
}

async fn handshake_verack(
    stream: &mut TcpStream,
    start_string: String,
) -> Result<(), error::Error> {
    // Send verack message
    let mut msg = Verack::new(start_string);
    info!("Sending verack message");
    debug!("Message data: {:#x?}", msg);
    let serialized_msg = msg.to_bytes()?;
    stream.write_all(&serialized_msg).await?;

    // Receive verack
    let reader = BufReader::new(serialized_msg.as_slice());
    let msg1 = Verack::from_bytes(reader).await?;
    info!("Receive verack message");
    debug!("Message data: {:#x?}", msg1);
    Ok(())
}

async fn handshake_version(
    stream: &mut TcpStream,
    socket: &mut SocketAddr,
    start_string: String,
) -> Result<(), error::Error> {
    let mut recv = stream.local_addr()?;

    to_ipv6(&mut recv);
    to_ipv6(socket);

    // Send version message
    let mut send_version = Version::new(start_string.clone(), generate_nonce(), *socket, recv)?;
    info!("Sending version message");
    debug!("Message data: {:#x?}", send_version);
    let serialized_msg = send_version.to_bytes()?;
    stream.write_all(&serialized_msg).await?;

    // Receive version
    let reader = BufReader::new(stream);

    let received_version = Version::from_bytes(reader).await?;
    info!("Received version messasge");
    debug!("Message data: {:#x?}", received_version);

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
