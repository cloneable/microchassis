#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    microchassis::init().await.map_err(Into::into)
}
