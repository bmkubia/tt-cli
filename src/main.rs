mod app;
mod client;
mod commands;
mod config;
mod interaction;
mod loader;
mod models;
mod version;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    app::run().await
}
