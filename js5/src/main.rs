use crate::config::Js5Config;
use crate::server::Js5Server;
use crate::service::Js5Service;
use filesystem::CacheBuilder;

mod error;
mod handshake;
mod config;
mod connection;
mod server;
mod service;
mod request;
mod response;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cache = CacheBuilder::new("cache/").open()?;
    let service = Js5Service::new(cache)?;
    let config = Js5Config::default();
    let server = Js5Server::new(config, service);

    println!("Starting Js5 server...");
    println!("Press Ctrl+C to quit...\n");

    server
        .run_until(async {
            tokio::signal::ctrl_c().await.ok();
        })
        .await?;

    Ok(())
}
