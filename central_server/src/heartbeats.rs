use std::net::SocketAddr;

use anyhow::Result;
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
};

async fn heartbeat_connection(mut socket: TcpStream, addr: SocketAddr) {
    let mut msg = String::new();
    match socket.read_to_string(&mut msg).await {
        Err(e) => {
            log::error!("failed to read from socket; err = {:?}", e);
            return;
        }
        Ok(_) => {}
    };

    log::info!("Msg ({}): {}", addr, msg);
}

pub async fn server() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9000").await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(heartbeat_connection(socket, addr));
    }
}
