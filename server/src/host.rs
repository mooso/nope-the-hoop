use std::{collections::HashMap, time::Duration};

use anyhow::{anyhow, Context};
use futures::{future::select_all, StreamExt};
use nope_the_hoop_proto::{
    message::{HorizontalDirection, ToClientMessage, ToServerMessage},
    state::{GameState, Point, UpdateState},
    stream::{write_message, MessageStream},
};
use tokio::{
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
    sync::mpsc,
    time::{Instant, MissedTickBehavior},
};
use tracing::{error, info, trace};

pub(crate) type ServerMessageStream = MessageStream<OwnedReadHalf, ToServerMessage>;

const INITIAL_HOOP_X: f32 = 100.;
const HOOP_MIN_X: f32 = 0.;
const HOOP_MAX_X: f32 = 200.;
const HOOOP_SPEED: f32 = 100.;
const SINGLE_BALL_POSITION: Point = Point { x: -100., y: 10. };
const BALL_SPEED_PER_SECOND_PRESSED: f32 = 100.;
const BALL_MAX_SPEED: f32 = 100.;
const FRAME_DURATION: Duration = Duration::from_millis(16);
const GRAVITY: f32 = 9.81;

pub struct GameHost {
    connection_tx: mpsc::Sender<(ServerMessageStream, OwnedWriteHalf)>,
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

    pub async fn new_client(&self, read_wrap: ServerMessageStream, write: OwnedWriteHalf) {
        self.connection_tx
            .send((read_wrap, write))
            .await
            .expect("Sending new client");
    }
}

struct Client {
    read: ServerMessageStream,
    write: OwnedWriteHalf,
}

async fn read_one_client_message(
    clients: &mut [Client],
) -> (usize, anyhow::Result<ToServerMessage>) {
    if clients.is_empty() {
        let () = futures::future::pending().await;
        unreachable!()
    }
    let (result, client_index, _) = select_all(
        clients
            .iter_mut()
            .map(|client| Box::pin(client.read.next())),
    )
    .await;
    let result = result.unwrap_or(Err(anyhow!("Client closed connection")));
    (client_index, result)
}

async fn game_loop(
    mut connection_rx: mpsc::Receiver<(ServerMessageStream, OwnedWriteHalf)>,
    id: u32,
) -> anyhow::Result<()> {
    let mut game = GameState {
        hoop_x: INITIAL_HOOP_X,
        ball_positions: HashMap::new(),
    };
    let mut ball_velocities = HashMap::new();
    game.ball_positions.insert(0, SINGLE_BALL_POSITION);
    ball_velocities.insert(0, None);
    let mut clients: Vec<Client> = vec![];
    let mut frame_timer = tokio::time::interval(FRAME_DURATION);
    frame_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut last_frame_time = Instant::now();
    loop {
        let mut updates = vec![];
        tokio::select! {
            new_connection = connection_rx.recv() => {
                let (read, mut write) = new_connection.context("Failed to receive connection")?;
                write_message(&mut write, &ToClientMessage::InitialState(game.clone())).await?;
                let mut client = Client { read, write };
                if clients.is_empty() {
                    info!("Game {} has its first client", id);
                    write_message(&mut client.write, &ToClientMessage::EstablishAsHoop).await?;
                } else if clients.len() == 1 {
                    info!("Game {} has its first ball", id);
                    write_message(&mut client.write, &ToClientMessage::EstablishAsBall {
                        id: 0,
                    }).await?;
                }
                clients.push(client);
            }
            (client_index, result) = read_one_client_message(&mut clients) => {
                let message = match result {
                    Ok(message) => message,
                    Err(e) => {
                        info!("Client {} in game {id} read error (terminating): {:#}", client_index, e);
                        _ = clients.remove(client_index);
                        continue;
                    }
                };
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
                    ToServerMessage::ShootBall {
                        id,
                        angle,
                        seconds_pressed,
                    } => {
                        trace!("Client {} in game {id} shot ball: {:?}", client_index, id);
                        let velocity = calculate_ball_velocity(angle, seconds_pressed);
                        ball_velocities.insert(id, Some(velocity));

                    }
                    ToServerMessage::Hello { .. } => {
                        error!("Client {} in game {id} sent Hello after initial hello - terminating", client_index);
                        _ = clients.remove(client_index);
                    }
                }
            }
            _ = frame_timer.tick() => {
                let now = Instant::now();
                let elapsed = now - last_frame_time;
                last_frame_time = now;
                for (id, velocity) in ball_velocities.iter_mut() {
                    let Some(velocity) = velocity else {
                        continue;
                    };
                    let Some(ball) = game.ball_positions.get_mut(id) else {
                        continue;
                    };
                    ball.x += velocity.x * elapsed.as_secs_f32();
                    ball.y += velocity.y * elapsed.as_secs_f32();
                    updates.push(ToClientMessage::UpdateState(UpdateState::MoveBall {
                        id: *id,
                        position: *ball,
                    }));
                    velocity.y -= GRAVITY * elapsed.as_secs_f32();
                }
            }
        }
        for update in updates {
            // TODO: Update concurrently, and don't let a slow client slow everyone down
            for client in &mut clients {
                trace!("Sending update to client: {update:?}");
                write_message(&mut client.write, &update).await?;
            }
        }
    }
}

fn calculate_ball_velocity(angle: f32, seconds_pressed: f32) -> Point {
    let speed = (seconds_pressed * BALL_SPEED_PER_SECOND_PRESSED).clamp(0., BALL_MAX_SPEED);
    let x = angle.cos() * speed;
    let y = angle.sin() * speed;
    Point { x, y }
}
