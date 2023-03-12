use std::{
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use log::{debug, trace};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

use crate::{error, protocol::messages::digest};

use super::{bytes_to_socket, socket_to_bytes, Header, Message};

#[derive(Debug)]
struct VersionPayload {
    version: i32,
    services: u64,
    timestamp: i64,
    addr_recv_services: u64,
    recv: SocketAddr, // ip and port
    addr_trans_services: u64,
    trans: SocketAddr, // ip and port
    nonce: u64,
    user_agent_bytes: u8,
    user_agent: Option<Vec<u8>>,
    start_height: i32,
    relay: Option<u8>,
}

#[derive(Debug)]
pub struct Version {
    // Header part of message
    header: Header,
    // Payload part of message
    payload: VersionPayload,
}

impl Version {
    pub fn new(
        start_string: String,
        nonce: u64,
        trans: SocketAddr,
        recv: SocketAddr,
    ) -> error::Result<Self> {
        let name = b"version\0\0\0\0\0";
        let header = Header {
            start_string,
            command_name: name.to_vec(),
            payload_size: 85,
            checksum: Vec::new(),
        };
        let time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let payload = VersionPayload {
            version: 70016,
            services: 1,
            timestamp: time as i64,
            addr_recv_services: 1,
            recv,
            addr_trans_services: 1,
            trans,
            nonce,
            user_agent_bytes: 0,
            user_agent: None,
            start_height: 0,
            relay: None,
        };
        Ok(Self { header, payload })
    }
    fn user_agent_to_bytes(&self) -> Vec<u8> {
        match &self.payload.user_agent {
            Some(agent) => agent.clone(),
            None => Vec::new(),
        }
    }
    pub fn get_nonce(&self)->u64{
        self.payload.nonce
    }
}
#[async_trait]
impl Message for Version {
    fn to_bytes(&mut self) -> error::Result<Vec<u8>> {
        let mut payload: Vec<u8> = Vec::new();
        payload.extend(self.payload.version.to_le_bytes());
        payload.extend(self.payload.services.to_le_bytes());
        payload.extend(self.payload.timestamp.to_le_bytes());
        payload.extend(self.payload.addr_recv_services.to_le_bytes());
        payload.extend(socket_to_bytes(self.payload.recv));
        payload.extend(self.payload.addr_trans_services.to_le_bytes());
        payload.extend(socket_to_bytes(self.payload.trans));
        payload.extend(self.payload.nonce.to_le_bytes());
        payload.extend(self.payload.user_agent_bytes.to_le_bytes());
        payload.extend(self.user_agent_to_bytes());
        payload.extend(self.payload.start_height.to_le_bytes());
        if let Some(relay) = self.payload.relay {
            payload.extend(relay.to_le_bytes());
        }

        let checksum = digest(payload.as_slice());
        self.header.update_checksum(checksum);
        trace!("New checksum: {:#x?}", self.header.checksum);

        let mut header = self.header.to_bytes()?;
        header.extend(payload.iter());
        Ok(header)
    }

    async fn from_bytes<T>(mut input: BufReader<T>) -> error::Result<Self>
    where
        T: AsyncRead + Unpin + Send,
        Self: Sized,
    {
        let header = Header::from_bytes(&mut input).await?;
        let mut payload_vec: Vec<u8> = Vec::new();
        // Read payload bytes
        for _ in 0..header.payload_size {
            payload_vec.push(input.read_u8().await?);
        }
        
        // Checksum check
        let checksum = digest(payload_vec.as_slice());
        let matches = checksum
            .iter()
            .zip(header.checksum.iter())
            .filter(|elem| elem.0 != elem.1)
            .count();
        if matches > 0 {
            return Err(error::Error::ChecksumError);
        }

        let mut input = BufReader::new(payload_vec.as_slice());


        let version = input.read_i32_le().await?;
        let services = input.read_u64_le().await?;
        let timestamp = input.read_i64_le().await?;
        let addr_recv_services = input.read_u64_le().await?;
        let recv_addr = &mut [0u8; 16];
        input.read_exact(recv_addr).await?;
        let recv_port = input.read_u16().await?;
        let recv = bytes_to_socket(recv_addr, recv_port);
        let addr_trans_services = input.read_u64_le().await?;
        let trans_addr = &mut [0u8; 16];
        input.read_exact(trans_addr).await?;
        let trans_port = input.read_u16().await?;
        let trans = bytes_to_socket(trans_addr, trans_port);
        let nonce = input.read_u64_le().await?;
        let user_agent_bytes = input.read_u8().await?;
        let user_agent = if user_agent_bytes > 0 {
            let mut user_agent: Vec<u8> = Vec::new();
            for _ in 0..user_agent_bytes {
                let byte = input.read_u8().await?;
                user_agent.push(byte);
            }
            Some(user_agent)
        } else {
            None
        };
        let start_height = input.read_i32_le().await?;

        let diff = header.payload_size - (85 + user_agent_bytes as u32);
        let relay = if diff == 1 {
            let relay = input.read_u8().await?;
            Some(relay)
        } else {
            None
        };

        let payload = VersionPayload {
            version,
            services,
            timestamp,
            addr_recv_services,
            recv,
            addr_trans_services,
            trans,
            nonce,
            user_agent_bytes,
            user_agent,
            start_height,
            relay,
        };
        Ok(Self { header, payload })
    }
}
