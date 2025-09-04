mod core;
mod strategy;

use std::env::args;
use std::path::PathBuf;
use strategy::get_strategy;
use core::ipc::EngineChannel;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{:?}", e.context("a fatal error occured"));
    }
}

async fn run() -> anyhow::Result<()> {
    let args = args();
    if args.len() < 2 {
        println!("usage: [bin name] [shmem path]");
        return Ok(());
    }

    let path = PathBuf::from(&args.skip(1).next().unwrap());
    let chan = EngineChannel::from_path(path)?;

    chan.handle_handshake().await?;

    let strat = get_strategy();

    loop {
        chan.handle_msg(&strat).await;
    }
}
