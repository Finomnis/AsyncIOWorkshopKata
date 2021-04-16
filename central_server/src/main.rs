mod connections;
mod heartbeats;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging subsystem
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Execute tasks
    tokio::try_join!(heartbeats::server(), connections::websocket_server())?;

    // All tasks finished, shouldn't happen usually as the tasks run indefinitely
    Ok(())
}
