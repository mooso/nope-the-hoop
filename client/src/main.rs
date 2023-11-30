use std::{fmt::Display, net::TcpStream};

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use clap::Parser;
use nope_the_hoop_proto::{
    message::{HorizontalDirection, ToClientMessage, ToServerMessage},
    state::{GameState, Point, UpdateState},
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
}

#[derive(Component)]
struct Hoop;

#[derive(Component)]
struct Ball;

#[derive(Resource)]
struct ServerConnection(MessageStream<TcpStream>);

#[derive(Resource)]
struct CurrentRole(Role);

#[derive(Resource)]
struct AssetHandles {
    hoop_mesh: Mesh2dHandle,
    hoop_material: Handle<ColorMaterial>,
    ball_mesh: Mesh2dHandle,
    ball_material: Handle<ColorMaterial>,
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
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(
            Startup,
            (setup_connect, setup_view, setup_role, setup_assets),
        )
        .add_systems(Update, (update_from_server, handle_input))
        .run();
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
    let hoop_material = materials.add(ColorMaterial::from(Color::GRAY));
    let hoop_mesh = meshes
        .add(shape::Quad::new(Vec2::new(50., 10.)).into())
        .into();
    let ball_material = materials.add(ColorMaterial::from(Color::RED));
    let ball_mesh = meshes
        .add(
            shape::Circle {
                radius: 10.,
                ..default()
            }
            .into(),
        )
        .into();
    commands.insert_resource(AssetHandles {
        hoop_mesh,
        hoop_material,
        ball_mesh,
        ball_material,
    });
}

fn update_from_server(
    mut commands: Commands,
    mut server: ResMut<ServerConnection>,
    mut current_role: ResMut<CurrentRole>,
    asset_handles: Res<AssetHandles>,
    mut hoops: Query<(&Hoop, &mut Transform)>,
) {
    let messages = server.0.read_messages::<ToClientMessage>().handle();
    for message in messages {
        match message {
            ToClientMessage::EstablishAsHoop => {
                trace!("I'm a hoop");
                current_role.0 = Role::Hoop;
            }
            ToClientMessage::UpdateState(UpdateState::MoveHoop { x }) => {
                hoops.single_mut().1.translation.x = x;
            }
            ToClientMessage::UpdateState(UpdateState::AddBall { position }) => {
                add_ball(&mut commands, position, &asset_handles);
            }
            ToClientMessage::InitialState(GameState {
                hoop_x,
                ball_positions,
            }) => {
                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: asset_handles.hoop_mesh.clone(),
                        material: asset_handles.hoop_material.clone(),
                        transform: Transform::from_translation(Vec3::new(hoop_x, 0., 0.)),
                        ..default()
                    },
                    Hoop,
                ));
                for ball in ball_positions {
                    add_ball(&mut commands, ball, &asset_handles);
                }
            }
        }
    }
}

fn add_ball(commands: &mut Commands, position: Point, asset_handles: &AssetHandles) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: asset_handles.ball_mesh.clone(),
            material: asset_handles.ball_material.clone(),
            transform: Transform::from_translation(Vec3::new(position.x, position.y, 0.)),
            ..default()
        },
        Ball,
    ));
}

fn handle_input(
    mut server: ResMut<ServerConnection>,
    current_role: Res<CurrentRole>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    match current_role.0 {
        Role::Unknown => {}
        Role::Hoop => {
            if keyboard_input.pressed(KeyCode::Left) {
                send_hoop_movement(&mut server, HorizontalDirection::Left, &time);
            }
            if keyboard_input.pressed(KeyCode::Right) {
                send_hoop_movement(&mut server, HorizontalDirection::Right, &time);
            }
        }
    }
}

fn establish_connection(args: &Args) -> anyhow::Result<MessageStream<TcpStream>> {
    let stream = TcpStream::connect((args.server.as_str(), args.port))?;
    stream.set_nonblocking(true)?;
    Ok(MessageStream::new(stream))
}

fn send_hoop_movement(server: &mut ServerConnection, direction: HorizontalDirection, time: &Time) {
    server
        .0
        .write_message(&ToServerMessage::MoveHoop {
            direction,
            seconds_pressed: time.delta_seconds(),
        })
        .handle();
}

fn send_hello(server: &mut ServerConnection) {
    server
        .0
        .write_message(&ToServerMessage::Hello { game_id: 123 })
        .handle();
}
