use anyhow::{anyhow, Result};
use std::{collections::HashSet, net::IpAddr};
use tokio::sync::{mpsc, watch};

use crate::heartbeats::HeartbeatProvider;

pub struct AgentTracker {
    new_connections: HashSet<IpAddr>,
    connections: HashSet<IpAddr>,
    heartbeats: mpsc::Receiver<IpAddr>,
    heartbeat_sender: mpsc::Sender<IpAddr>,
    agents: watch::Sender<HashSet<IpAddr>>,
    agents_listener: watch::Receiver<HashSet<IpAddr>>,
}

impl AgentTracker {
    pub fn new() -> Self {
        let (heartbeat_sender, heartbeats) = mpsc::channel(1);
        let (agents, agents_listener) = watch::channel(HashSet::new());
        Self {
            new_connections: HashSet::new(),
            connections: HashSet::new(),
            heartbeats,
            heartbeat_sender,
            agents,
            agents_listener,
        }
    }

    pub fn create_heartbeat_provider(&self) -> HeartbeatProvider {
        HeartbeatProvider::new(self.heartbeat_sender.clone())
    }

    async fn process_heartbeats(&mut self) -> Result<()> {
        loop {
            let agent_addr = self
                .heartbeats
                .recv()
                .await
                .ok_or(anyhow!("Heartbeat mpsc closed!"))?;

            log::info!("Received heartbeat from {}", agent_addr);
            self.new_connections.insert(agent_addr);
            if self.connections.insert(agent_addr) {
                self.agents.send(self.connections.clone())?;
            }
        }
    }

    pub async fn main(mut self) -> Result<()> {
        tokio::try_join!(self.process_heartbeats())?;
        Ok(())
    }
}
