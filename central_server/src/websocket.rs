use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::WatchStream;
use tokio_tungstenite::tungstenite::{error::Error as WsError, Message};

use crate::agent_tracker::ConnectedAgentsWatch;

async fn connection(socket: TcpStream, addr: SocketAddr, agents_watch: ConnectedAgentsWatch) {
    let ws_stream = match tokio_tungstenite::accept_async(socket).await {
        Ok(ws) => ws,
        Err(err) => {
            log::error!("Unable to initiate websocket connection: {}", err);
            return;
        }
    };

    log::info!("{} connected", addr);
    let (mut write, mut read) = ws_stream.split();

    let receive_task = async move {
        loop {
            let next = read.next().await;
            match next {
                None => break,
                Some(Err(WsError::ConnectionClosed)) => break,
                Some(Err(err)) => {
                    log::error!("Connection to {} closed abnormally: {}", addr.ip(), err);
                    break;
                }
                Some(Ok(_)) => (),
            };
        }
    };

    let send_task = async move {
        let mut agents = WatchStream::new(agents_watch.receiver);
        loop {
            let msg = match agents.next().await {
                Some(m) => m,
                None => {
                    log::error!("Watch channel closed!");
                    break;
                }
            };

            if let Err(err) = write.send(Message::Text(msg)).await {
                log::error!("Unable to write to websocket: {}", err);
                break;
            };
        }
    };

    tokio::select! {
        _ = send_task => (),
        _ = receive_task => (),
    };

    log::info!("{} disconnected", addr);
}

pub async fn server(agents_watch: ConnectedAgentsWatch) -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:9001").await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(connection(socket, addr, agents_watch.clone()));
    }
}
