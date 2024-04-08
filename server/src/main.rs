use std::{collections::HashMap, time::Duration};

use anyhow::Context;
use clap::Parser;
use futures::{future::select_all, StreamExt};
use host::ServerMessageStream;
use nope_the_hoop_proto::{message::ToServerMessage, stream::MessageStream};
use tokio::net::TcpListener;
use tracing::info;

use crate::host::GameHost;

mod host;
mod sim;

#[derive(Parser)]
#[command(
    author = "Mostafa",
    version = "0",
    about = "Server for the hit nope-the-hoop game"
)]
struct Args {
    /// The port to bind to.
    #[arg(short, long, default_value_t = 7434)]
    port: u16,

    /// The address to bind to.
    #[arg(long, default_value = "127.0.0.1")]
    bind_address: String,
}

#[tokio::main]
async fn main() {
    let _guard =
        tracing::subscriber::set_global_default(tracing_subscriber::fmt::Subscriber::new());
    let args = Args::parse();
    let listener = TcpListener::bind(&format!("{}:{}", args.bind_address, args.port))
        .await
        .unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    let mut games: HashMap<u32, GameHost> = HashMap::new();

    loop {
        tokio::select! {
            result = listener.accept() => {
                let (stream, addr) = result.expect("Accepting connection");
                info!("Accepted connection from {}", addr);
                let (read, write) = stream.into_split();
                let mut read = MessageStream::new(read);
                let game_id = match process_hello(&mut read).await {
                    Ok(game_id) => game_id,
                    Err(e) => {
                        info!("Connection from {} failed on hello: {:#}", addr, e);
                        return;
                    }
                };
                let game = games.entry(game_id).or_insert_with(|| GameHost::new(game_id));
                game.new_client(read, write).await;
            }
            ended_game = await_game_end(&mut games) => {
                games.remove(&ended_game);
            }
        }
    }
}

async fn await_game_end(games: &mut HashMap<u32, GameHost>) -> u32 {
    if games.is_empty() {
        let () = futures::future::pending().await;
        unreachable!()
    }
    let (ended_game, _, _) =
        select_all(games.values_mut().map(|game| Box::pin(game.await_end()))).await;
    ended_game
}

async fn process_hello(read: &mut ServerMessageStream) -> anyhow::Result<u32> {
    let result = tokio::time::timeout(Duration::from_millis(500), read.next())
        .await
        .context("Timed out on receiving hello")?;
    let Some(result) = result else {
        anyhow::bail!("Client closed connection before hello");
    };
    let client_message = result.context("Failed to read hello")?;
    let ToServerMessage::Hello { game_id } = client_message else {
        anyhow::bail!("Expected Hello from client - got: {:?}", client_message);
    };
    Ok(game_id)
}
