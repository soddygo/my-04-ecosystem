#[allow(unused)]
mod chat_server;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    chat_server::chat_server().await?;

    Ok(())
}
