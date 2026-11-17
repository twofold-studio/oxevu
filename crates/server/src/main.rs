#[tokio::main]
async fn main() -> anyhow::Result<()> {
    oxevu_api::run().await?;
    Ok(())
}
