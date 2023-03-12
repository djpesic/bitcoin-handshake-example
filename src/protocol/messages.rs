
use async_trait::async_trait;
use serde_derive::{Deserialize, Serialize};
use tokio::{
    io::{AsyncRead, AsyncReadExt, BufReader},
    net::TcpStream,
};

use crate::error;
#[async_trait]
pub trait Message {
    fn to_bytes(&self) -> error::Result<Vec<u8>>;
    async fn from_bytes<T>(mut input: BufReader<T>) -> error::Result<Self>
    where
        T: AsyncRead + Unpin+Send,
        Self: Sized;
    
}

#[derive(Debug)]
struct Header {
    start_string: String,
    command_name: Vec<u8>,
    payload_size: u32,
    checksum: u32,
}

impl Header {
    const START_STRING_SIZE: u8 = 4;
    const COMMAND_NAME_SIZE: u8 = 12;
    const PAYLOAD_SIZE_SIZE: u8 = 4;
    const CHECKSUM_SIZE: u8 = 4;
    fn to_bytes(&self) -> error::Result<Vec<u8>> {
        let mut result: Vec<u8> = Vec::new();
        result.extend(self.from_hex_string(&self.start_string)?.iter());
        result.extend(self.command_name.iter());
        result.extend(self.payload_size.to_le_bytes().iter());
        result.extend(self.checksum.to_le_bytes().iter());
        Ok(result)
    }

    async fn from_bytes<T>(mut input: BufReader<T>) -> error::Result<Self>
    where
        T: AsyncRead + Unpin,
    {
        let start_string = &mut [0u8; 4];
        let command_name = &mut [0u8; 12];

        input.read_exact(start_string).await?;
        input.read_exact(command_name).await?;
        let payload_size = input.read_u32_le().await?;
        let checksum = input.read_u32_le().await?;

        Ok(Self {
            start_string: Header::to_hex_string(start_string.to_vec()),
            command_name: command_name.to_vec(),
            payload_size,
            checksum,
        })
    }

    fn from_hex_string(&self, hex_string: &String) -> error::Result<Vec<u8>> {
        let mut decoded = hex::decode(hex_string)?;
        decoded.reverse();
        Ok(decoded)
    }

    fn to_hex_string(bytes: Vec<u8>) -> String {
        let mut bytes = bytes;
        bytes.reverse();
        hex::encode(bytes)
    }
}

#[derive(Debug)]
pub struct Verack{
    header: Header,
}

impl Verack {
    pub fn new(start_string: String) -> Self {
        let name = b"verack\0\0\0\0\0\0";
        Self {
            header: Header {
                start_string,
                command_name: name.to_vec(),
                payload_size: 0,
                checksum: 0x5df6e0e2,
            },
        }
    }
}
#[async_trait]
impl Message for Verack {
    fn to_bytes(&self) -> error::Result<Vec<u8>> {
        self.header.to_bytes()
    }

    async fn from_bytes<T>(input: BufReader<T>) -> error::Result<Self>
    where
        T: AsyncRead + Unpin+Send,
        Self:Sized
        {
            let header = Header::from_bytes(input).await?;
            Ok(Self{header})
        }
}
