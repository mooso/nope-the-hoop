use anyhow::Context;
use futures::future::select_all;
use nope_the_hoop_proto::{
    read_messages_as_server,
    state::{GameState, UpdateState},
    write_message, HorizontalDirection, ToClientMessage, ToServerMessage,
};
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf, sync::mpsc};
use tracing::{error, info, trace};

use crate::sync_read::ReadWrap;

const INITIAL_HOOP_X: f32 = 100.;
const HOOP_MIN_X: f32 = 0.;
const HOOP_MAX_X: f32 = 200.;
const HOOOP_SPEED: f32 = 100.;

pub struct GameHost {
    connection_tx: mpsc::Sender<(ReadWrap, OwnedWriteHalf)>,
    end_rx: mpsc::Receiver<u32>,
}

impl GameHost {
    pub fn new(id: u32) -> Self {
        info!("Starting game {}", id);
        let (connection_tx, connection_rx) = mpsc::channel(4);
        let (end_tx, end_rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let result = game_loop(connection_rx, id).await;
            end_tx.send(id).await.expect("Sending end to a live server");
            if let Err(e) = result {
                error!("Game loop error for game {id}: {:#}", e);
            }
        });
        Self {
            connection_tx,
            end_rx,
        }
    }

    pub async fn await_end(&mut self) -> u32 {
        self.end_rx.recv().await.expect("Awaiting end of game")
    }

    pub async fn new_client(&self, read_wrap: ReadWrap, write: OwnedWriteHalf) {
        self.connection_tx
            .send((read_wrap, write))
            .await
            .expect("Sending new client");
    }
}

struct Client {
    read: ReadWrap,
    write: OwnedWriteHalf,
}

impl Client {
    async fn read_ready(&self) -> tokio::io::Result<()> {
        self.read.await_ready().await
    }
}

async fn read_one_client_message(
    clients: &mut [Client],
) -> (usize, anyhow::Result<Vec<ToServerMessage>>) {
    if clients.is_empty() {
        let () = futures::future::pending().await;
        unreachable!()
    }
    let (result, client_index, _) =
        select_all(clients.iter().map(|client| Box::pin(client.read_ready()))).await;
    (
        client_index,
        match result {
            Ok(()) => read_messages_as_server(&mut clients[client_index].read),
            Err(e) => Err(e.into()),
        },
    )
}

async fn game_loop(
    mut connection_rx: mpsc::Receiver<(ReadWrap, OwnedWriteHalf)>,
    id: u32,
) -> anyhow::Result<()> {
    let mut game = GameState {
        hoop_x: INITIAL_HOOP_X,
    };
    let mut clients: Vec<Client> = vec![];
    loop {
        let mut updates = vec![];
        tokio::select! {
            new_connection = connection_rx.recv() => {
                let (read, mut write) = new_connection.context("Failed to receive connection")?;
                write_to_client(&mut write, &ToClientMessage::InitialState(game.clone())).await?;
                let mut client = Client { read, write };
                if clients.is_empty() {
                    info!("Game {} has its first client", id);
                    write_to_client(&mut client.write, &ToClientMessage::EstablishAsHoop).await?;
                }
                clients.push(client);
            }
            (client_index, result) = read_one_client_message(&mut clients) => {
                let messages = match result {
                    Ok(messages) => messages,
                    Err(e) => {
                        info!("Client {} in game {id} read error (terminating): {:#}", client_index, e);
                        _ = clients.remove(client_index);
                        continue;
                    }
                };
                for message in messages {
                    match message {
                        ToServerMessage::MoveHoop {
                            direction,
                            seconds_pressed,
                        } => {
                            trace!("Client {} in game {id} moved hoop: {:?}", client_index, direction);
                            let sign = match direction {
                                HorizontalDirection::Left => -1.,
                                HorizontalDirection::Right => 1.,
                            };
                            let delta_x = sign * HOOOP_SPEED * seconds_pressed;
                            game.hoop_x = (game.hoop_x + delta_x).clamp(HOOP_MIN_X, HOOP_MAX_X);
                            updates.push(ToClientMessage::UpdateState(UpdateState::MoveHoop { x: game.hoop_x }));
                        }
                        ToServerMessage::Hello { .. } => {
                            error!("Client {} in game {id} sent Hello after initial hello - terminating", client_index);
                            _ = clients.remove(client_index);
                        }
                    }
                }
            }
        }
        for update in updates {
            // TODO: Update concurrently, and don't let a slow client slow everyone down
            for client in &mut clients {
                trace!("Sending update to client: {update:?}");
                write_to_client(&mut client.write, &update).await?;
            }
        }
    }
}

async fn write_to_client(
    write: &mut OwnedWriteHalf,
    command: &ToClientMessage,
) -> anyhow::Result<()> {
    let mut buf = vec![];
    write_message(&mut buf, command)?;
    write.write_all(&buf).await?;
    Ok(())
}
