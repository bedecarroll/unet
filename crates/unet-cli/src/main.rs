use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    unet_cli::run(std::env::args_os()).await
}
