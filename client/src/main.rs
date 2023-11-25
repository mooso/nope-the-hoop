use std::{fmt::Display, net::TcpStream};

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use clap::Parser;
use nope_the_hoop_proto::{
    read_messages_as_client, write_message, HorizontalDirection, ToClientMessage, ToServerMessage,
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

#[derive(Resource)]
struct ServerConnection(TcpStream);

#[derive(Resource)]
struct CurrentRole(Role);

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
        .add_systems(Startup, (setup_connect, setup_view, setup_role))
        .add_systems(Update, (update_from_server, handle_input))
        .run();
}

fn setup_connect(mut commands: Commands) {
    let args = Args::parse();
    info!("Connecting to {}:{}", args.server, args.port);
    let stream = establish_connection(&args).handle();
    commands.insert_resource(ServerConnection(stream));
    info!("Connected");
}

fn setup_view(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_role(mut commands: Commands) {
    commands.insert_resource(CurrentRole(Role::Unknown));
}

fn update_from_server(
    mut commands: Commands,
    mut server: ResMut<ServerConnection>,
    mut current_role: ResMut<CurrentRole>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hoops: Query<(&Hoop, &mut Transform)>,
) {
    let messages = read_messages_as_client(&mut server.0).handle();
    for message in messages {
        match message {
            ToClientMessage::EstablishAsHoop { x } => {
                current_role.0 = Role::Hoop;
                commands.spawn((
                    MaterialMesh2dBundle {
                        mesh: meshes
                            .add(shape::Quad::new(Vec2::new(50., 10.)).into())
                            .into(),
                        material: materials.add(ColorMaterial::from(Color::GRAY)),
                        transform: Transform::from_translation(Vec3::new(x, 0., 0.)),
                        ..default()
                    },
                    Hoop,
                ));
            }
            ToClientMessage::MoveHoop { x } => {
                hoops.single_mut().1.translation.x = x;
            }
        }
    }
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

fn establish_connection(args: &Args) -> anyhow::Result<TcpStream> {
    let stream = TcpStream::connect((args.server.as_str(), args.port))?;
    stream.set_nonblocking(true)?;
    Ok(stream)
}

fn send_hoop_movement(server: &mut ServerConnection, direction: HorizontalDirection, time: &Time) {
    write_message(
        &mut server.0,
        &ToServerMessage::MoveHoop {
            direction,
            seconds_pressed: time.delta_seconds(),
        },
    )
    .handle();
}
