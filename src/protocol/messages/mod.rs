pub mod verack;
pub mod version;

use std::net::{IpAddr, Ipv6Addr, SocketAddr};

use async_trait::async_trait;
use sha2::Digest;
use sha2::Sha256;
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

use crate::error;
#[async_trait]
pub trait Message {
    fn to_bytes(&mut self) -> error::Result<Vec<u8>>;

    async fn from_bytes<T>(mut input: BufReader<T>) -> error::Result<Self>
    where
        T: AsyncRead + Unpin + Send,
        Self: Sized;
}

#[derive(Debug)]
struct Header {
    start_string: String,
    command_name: Vec<u8>,
    payload_size: u32,
    checksum: Vec<u8>,
}

impl Header {
    fn to_bytes(&self) -> error::Result<Vec<u8>> {
        let mut result: Vec<u8> = Vec::new();
        result.extend(from_hex_string(&self.start_string)?.iter());
        result.extend(self.command_name.iter());
        result.extend(self.payload_size.to_le_bytes().iter());
        result.extend(self.checksum.iter());
        Ok(result)
    }

    async fn from_bytes<T>(input: &mut BufReader<T>) -> error::Result<Self>
    where
        T: AsyncRead + Unpin,
    {
        let start_string = &mut [0u8; 4];
        let command_name = &mut [0u8; 12];
        input.read_exact(start_string).await?;
        input.read_exact(command_name).await?;
        let payload_size = input.read_u32_le().await?;
        let checksum = &mut [0u8; 4];
        input.read_exact(checksum).await?;

        let start_string = start_string.to_vec();

        let checksum = checksum.to_vec();
        Ok(Self {
            start_string: to_hex_string(start_string),
            command_name: command_name.to_vec(),
            payload_size,
            checksum,
        })
    }
    fn update_checksum(&mut self, new_checksum: Vec<u8>) {
        self.checksum = new_checksum;
    }
}

pub fn from_hex_string(hex_string: &String) -> error::Result<Vec<u8>> {
    let decoded = hex::decode(hex_string)?;
    Ok(decoded)
}

pub fn to_hex_string(bytes: Vec<u8>) -> String {
    hex::encode(bytes)
}

fn socket_to_bytes(socket: SocketAddr) -> Vec<u8> {
    let mut result = match socket.ip() {
        IpAddr::V4(ip) => ip.octets().to_vec(),
        IpAddr::V6(ip) => ip.octets().to_vec(),
    };
    result.extend(socket.port().to_be_bytes());
    result
}

fn bytes_to_socket(addr: &mut [u8; 16], port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V6(Ipv6Addr::from(*addr)), port)
}

fn digest(data: impl AsRef<[u8]>) -> Vec<u8> {
    // First round
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();

    // Second round
    let mut hasher = Sha256::new();
    hasher.update(digest);
    let digest = hasher.finalize();
    let mut result = Vec::new();
    result.extend_from_slice(&digest[..4]);
    result
}
