use log::{debug, info};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::{
    error,
    protocol::messages::{Message, Verack},
};

pub async fn start_handshake(socket: std::net::SocketAddr, start_string: String) {
    let result = run_handshake(socket, start_string).await;
    if let Err(e) = result {
        log::error!("{:?}", e);
    }
}

async fn run_handshake(
    socket: std::net::SocketAddr,
    start_string: String,
) -> Result<(), error::Error> {
    info!("Starting handshake with {:?}", socket);
    // Connect to a remote node
    // let mut stream = TcpStream::connect(socket).await?;

    // Send version message
    let msg = Verack::new(start_string);
    debug!("Verack: {:?}", msg);
    let serialized_msg = msg.to_bytes()?;

    debug!("Serialized msg: {:#x?}", serialized_msg);

    // stream.write_all(&serialized_msg).await?;

    let reader = BufReader::new(serialized_msg.as_slice());
    let msg1 =Verack::from_bytes(reader).await?;
    Ok(())
}

mod test{
    use tokio::io::BufReader;
    use crate::protocol::messages::Message;
    use crate::protocol::messages::Verack;

    #[tokio::test]
    async fn test_serde(){
        let msg = Verack::new(String::from("f9beb4d9"));
        let serialized_msg = msg.to_bytes().unwrap();
        let reader = BufReader::new(serialized_msg.as_slice());
        let msg1 =Verack::from_bytes(reader).await.unwrap();
        let str1 = format!("{:?}", msg);
        let str2 = format!("{:?}", msg1);
        assert_eq!(str1, str2);
    }
}
