mod agent_tracker;
mod heartbeats;
mod websocket;

use agent_tracker::AgentTracker;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging subsystem
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Initialize agent tracker object
    let tracker = AgentTracker::new();
    let heartbeat_provider = tracker.create_heartbeat_provider();

    // Execute tasks
    tokio::try_join!(
        heartbeats::server(heartbeat_provider),
        websocket::server(),
        tracker.main()
    )?;

    // All tasks finished, shouldn't happen usually as the tasks run indefinitely
    Ok(())
}
