use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tt::app::run().await
}
