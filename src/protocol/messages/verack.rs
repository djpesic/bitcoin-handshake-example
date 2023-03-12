use async_trait::async_trait;
use tokio::io::{AsyncRead, BufReader};

use crate::error;

use super::{digest, Header, Message};

#[derive(Debug)]
pub struct Verack {
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
                checksum: digest([]),
            },
        }
    }
}
#[async_trait]
impl Message for Verack {
    fn to_bytes(&mut self) -> error::Result<Vec<u8>> {
        self.header.to_bytes()
    }

    async fn from_bytes<T>(mut input: BufReader<T>) -> error::Result<Self>
    where
        T: AsyncRead + Unpin + Send,
        Self: Sized,
    {
        let header = Header::from_bytes(&mut input).await?;
        Ok(Self { header })
    }
}
