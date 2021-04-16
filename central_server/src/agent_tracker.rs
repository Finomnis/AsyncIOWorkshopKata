use anyhow::{anyhow, Result};
use serde_json as json;
use std::{
    collections::HashSet,
    mem::take,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{mpsc, watch, Mutex as AsyncMutex};

use crate::heartbeats::HeartbeatProvider;

struct Connections {
    current: HashSet<IpAddr>,
    fresh: HashSet<IpAddr>,
}

impl Connections {
    pub fn new() -> Self {
        Self {
            current: HashSet::new(),
            fresh: HashSet::new(),
        }
    }
}

#[derive(Clone)]
pub struct ConnectedAgentsWatch {
    pub receiver: watch::Receiver<String>,
}

pub struct AgentTracker {
    connections: Arc<Mutex<Connections>>,
    heartbeats: Arc<AsyncMutex<mpsc::Receiver<IpAddr>>>,
    heartbeat_sender: mpsc::Sender<IpAddr>,
    agents: watch::Sender<String>,
    agents_listener: watch::Receiver<String>,
}

impl AgentTracker {
    pub fn new() -> Self {
        let (heartbeat_sender, heartbeats) = mpsc::channel(1);
        let (agents, agents_listener) = watch::channel(json::json!([]).to_string());
        Self {
            connections: Arc::new(Mutex::new(Connections::new())),
            heartbeats: Arc::new(AsyncMutex::new(heartbeats)),
            heartbeat_sender,
            agents,
            agents_listener,
        }
    }

    pub fn create_heartbeat_provider(&self) -> HeartbeatProvider {
        HeartbeatProvider::new(self.heartbeat_sender.clone())
    }

    pub fn create_connected_agents_watch(&self) -> ConnectedAgentsWatch {
        ConnectedAgentsWatch {
            receiver: self.agents_listener.clone(),
        }
    }

    fn publish_new_agents_list(&self, connections: &HashSet<IpAddr>) -> Result<()> {
        let payload_raw = json::Value::Array(
            connections
                .iter()
                .map(|val| json::Value::String(val.to_string()))
                .collect(),
        );
        let payload = serde_json::to_string(&payload_raw)?;
        self.agents.send(payload)?;
        Ok(())
    }

    async fn process_heartbeats(&self) -> Result<()> {
        loop {
            let agent_addr = self
                .heartbeats
                .lock()
                .await
                .recv()
                .await
                .ok_or(anyhow!("Heartbeat mpsc closed!"))?;

            log::info!("Received heartbeat from {}", agent_addr);
            let mut connections = self
                .connections
                .lock()
                .or_else(|err| Err(anyhow!("Unable to lock connections: {}", err)))?;

            connections.fresh.insert(agent_addr);
            if connections.current.insert(agent_addr) {
                self.publish_new_agents_list(&connections.current)?;
                log::info!("Connections changed: {:?}", connections.current);
            }
        }
    }

    async fn expire_heartbeats(&self) -> Result<()> {
        loop {
            tokio::time::sleep(Duration::from_secs(3)).await;
            let mut connections = self
                .connections
                .lock()
                .or_else(|err| Err(anyhow!("Unable to lock connections: {}", err)))?;

            let needs_update = connections.current != connections.fresh;

            connections.current = take(&mut connections.fresh);

            if needs_update {
                self.publish_new_agents_list(&connections.current)?;
                log::info!("Connections changed: {:?}", connections.current);
            }
        }
    }

    pub async fn main(self) -> Result<()> {
        tokio::try_join!(self.process_heartbeats(), self.expire_heartbeats())?;
        Ok(())
    }
}
