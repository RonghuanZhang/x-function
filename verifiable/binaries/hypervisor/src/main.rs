use std::path::PathBuf;

use clap::Parser;
use hypervisor::{Config, Server};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Compute node
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = Args::parse();
    let config: Config = {
        match args.config {
            Some(path) => {
                let config_str = tokio::fs::read_to_string(path).await?;
                toml::from_str(&config_str)?
            }
            None => Config::default(),
        }
    };

    let server = Server::build(config)?;

    server.start().await
}
