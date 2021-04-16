use std::net::{IpAddr, SocketAddr};

use anyhow::Result;
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    sync::mpsc,
};

#[derive(Clone)]
pub struct HeartbeatProvider {
    heartbeat_sender: mpsc::Sender<IpAddr>,
}

impl HeartbeatProvider {
    pub fn new(heartbeat_sender: mpsc::Sender<IpAddr>) -> Self {
        Self { heartbeat_sender }
    }

    pub async fn send_heartbeat(&self, addr: IpAddr) -> Result<()> {
        Ok(self.heartbeat_sender.send(addr).await?)
    }
}

async fn heartbeat_connection(
    mut socket: TcpStream,
    addr: SocketAddr,
    heartbeat_provider: HeartbeatProvider,
) {
    let mut msg = String::new();
    match socket.read_to_string(&mut msg).await {
        Err(e) => {
            log::error!("failed to read from socket; err = {:?}", e);
            return;
        }
        Ok(_) => {}
    };

    if msg == "Heartbeat!" {
        if let Err(err) = heartbeat_provider.send_heartbeat(addr.ip()).await {
            log::error!("Unable to register heartbeat: {}", err);
        }
    }

    log::info!("Msg ({}): {}", addr, msg);
}

pub async fn server(heartbeat_provider: HeartbeatProvider) -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9000").await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(heartbeat_connection(
            socket,
            addr,
            heartbeat_provider.clone(),
        ));
    }
}
