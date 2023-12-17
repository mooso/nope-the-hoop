mod ball;
mod hoop;

use std::{fmt::Display, net::TcpStream};

use ball::add_ball;
use bevy::prelude::*;
use clap::Parser;
use hoop::{add_hoop, move_hoop, HoopQuery};
use nope_the_hoop_proto::{
    message::{ToClientMessage, ToServerMessage},
    state::{GameState, UpdateState},
    sync::MessageStream,
};

#[derive(Parser)]
#[command(
    author = "Mostafa",
    version = "0",
    about = "The hit nope-the-hoop game"
)]
struct Args {
    /// The port to connect to.
    #[arg(short = 'p', long, default_value_t = 7434)]
    port: u16,

    /// The server address to connect to.
    #[arg(short, long, default_value = "127.0.0.1")]
    server: String,
}

enum Role {
    Unknown,
    Hoop,
    Ball { origin: Vec2 },
}

#[derive(Resource)]
struct ServerConnection(MessageStream<TcpStream>);

#[derive(Resource)]
struct CurrentRole(Role);

#[derive(Resource)]
struct AssetHandles {
    hoop_assets: hoop::AssetHandles,
    ball_assets: ball::AssetHandles,
}

trait HandleErrors {
    type Output;

    fn handle(self) -> Self::Output;
}

impl<R, E: Display> HandleErrors for Result<R, E> {
    type Output = R;

    fn handle(self) -> R {
        match self {
            Ok(r) => r,
            Err(e) => {
                error!("Fatal error: {e:#}");
                std::process::exit(1);
            }
        }
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (setup_connect, setup_view, setup_role, setup_assets),
        )
        .add_systems(Update, update_from_server);
    ball::setup(&mut app);
    hoop::setup(&mut app);
    app.run();
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

fn setup_view(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_role(mut commands: Commands) {
    commands.insert_resource(CurrentRole(Role::Unknown));
}

fn setup_assets(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let hoop_assets = hoop::AssetHandles::create(&mut materials, &mut meshes);
    let ball_assets = ball::AssetHandles::create(&mut materials, &mut meshes);
    commands.insert_resource(AssetHandles {
        hoop_assets,
        ball_assets,
    });
}

fn update_from_server(
    mut commands: Commands,
    mut server: ResMut<ServerConnection>,
    mut current_role: ResMut<CurrentRole>,
    asset_handles: Res<AssetHandles>,
    mut hoops: HoopQuery,
) {
    let messages = server.0.read_messages::<ToClientMessage>().handle();
    for message in messages {
        match message {
            ToClientMessage::EstablishAsHoop => {
                trace!("I'm a hoop");
                current_role.0 = Role::Hoop;
            }
            ToClientMessage::EstablishAsBall { origin } => {
                trace!("I'm a ball");
                current_role.0 = Role::Ball {
                    origin: Vec2::new(origin.x, origin.y),
                };
            }
            ToClientMessage::UpdateState(UpdateState::MoveHoop { x }) => {
                move_hoop(&mut hoops, x);
            }
            ToClientMessage::UpdateState(UpdateState::AddBall { position }) => {
                add_ball(&mut commands, position, &asset_handles.ball_assets);
            }
            ToClientMessage::InitialState(GameState {
                hoop_x,
                ball_positions,
            }) => {
                add_hoop(&mut commands, hoop_x, &asset_handles.hoop_assets);
                for ball in ball_positions {
                    add_ball(&mut commands, ball, &asset_handles.ball_assets);
                }
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
    server
        .0
        .write_message(&ToServerMessage::Hello { game_id: 123 })
        .handle();
}
