use anyhow::Result;
use futures_util::StreamExt;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};

async fn connection(socket: TcpStream, addr: SocketAddr) {
    let ws_stream = match tokio_tungstenite::accept_async(socket).await {
        Ok(ws) => ws,
        Err(err) => {
            log::error!("Unable to initiate websocket connection: {}", err);
            return;
        }
    };

    log::info!("Connected: {}", addr);

    let (write, read) = ws_stream.split();
    match read.forward(write).await {
        Ok(()) => {}
        Err(err) => log::warn!("Websocket closed with error: {}", err),
    };
}

pub async fn server() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9001").await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(connection(socket, addr));
    }
}
