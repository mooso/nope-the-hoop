use std::net::TcpStream;

use crate::{
    ball::{add_ball, move_ball, remove_ball, BallQuery},
    hoop::{add_hoop, move_hoop, HoopQuery},
};
use bevy::prelude::*;
use clap::Parser;
use nope_the_hoop_proto::{
    message::{ToClientMessage, ToServerMessage},
    state::{GameState, UpdateState},
    sync::MessageStream,
};

use crate::{Args, AssetHandles, CurrentRole, HandleErrors, Role};

#[derive(Resource)]
pub struct ServerConnection(MessageStream<TcpStream>);

impl ServerConnection {
    pub fn send(&mut self, message: ToServerMessage) {
        self.0.write_message(&message).handle();
    }
}

pub fn setup(app: &mut App) {
    app.add_systems(Startup, setup_connect)
        .add_systems(Update, update_from_server);
}

fn setup_connect(mut commands: Commands) {
    let args = Args::parse();
    info!("Connecting to {}:{}", args.server, args.port);
    let stream = establish_connection(&args).handle();
    let mut connection = ServerConnection(stream);
    send_hello(&mut connection);
    commands.insert_resource(connection);
    info!("Connected");
}

fn update_from_server(
    mut commands: Commands,
    mut server: ResMut<ServerConnection>,
    mut current_role: ResMut<CurrentRole>,
    asset_handles: Res<AssetHandles>,
    mut hoops: HoopQuery,
    mut balls: BallQuery,
) {
    let messages = server.0.read_messages::<ToClientMessage>().handle();
    for message in messages {
        match message {
            ToClientMessage::EstablishAsHoop => {
                trace!("I'm a hoop");
                current_role.0 = Role::Hoop;
            }
            ToClientMessage::EstablishAsBall { id } => {
                trace!("I'm a ball");
                current_role.0 = Role::Ball { id };
            }
            ToClientMessage::UpdateState(UpdateState::MoveHoop { x }) => {
                move_hoop(&mut hoops, x);
            }
            ToClientMessage::UpdateState(UpdateState::AddBall { id, position }) => {
                add_ball(&mut commands, id, position, &asset_handles.ball_assets);
            }
            ToClientMessage::UpdateState(UpdateState::RemoveBall { id }) => {
                remove_ball(&mut commands, id, &mut balls);
            }
            ToClientMessage::UpdateState(UpdateState::MoveBall { id, position }) => {
                move_ball(id, position, &mut balls);
            }
            ToClientMessage::InitialState(GameState {
                hoop_x,
                ball_positions,
            }) => {
                add_hoop(&mut commands, hoop_x, &asset_handles.hoop_assets);
                for (id, ball) in ball_positions {
                    add_ball(&mut commands, id, ball, &asset_handles.ball_assets);
                }
            }
            ToClientMessage::EstablishAsObserver => {
                trace!("I'm an observer");
                current_role.0 = Role::Observer;
            }
        }
    }
}

fn establish_connection(args: &Args) -> anyhow::Result<MessageStream<TcpStream>> {
    let stream = TcpStream::connect((args.server.as_str(), args.port))?;
    stream.set_nonblocking(true)?;
    Ok(MessageStream::new(stream))
}

fn send_hello(server: &mut ServerConnection) {
    server.send(ToServerMessage::Hello { game_id: 123 });
}
