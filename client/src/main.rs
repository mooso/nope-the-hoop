use std::net::TcpStream;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use clap::Parser;
use nope_the_hoop_proto::{read_commands, Command, Role};

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

#[derive(Component)]
struct Hoop;

#[derive(Resource)]
struct ServerConnection(TcpStream);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_connect, setup_view))
        .add_systems(Update, update_from_server)
        .run();
}

fn setup_connect(mut commands: Commands) {
    let args = Args::parse();
    info!("Connecting to {}:{}", args.server, args.port);
    let stream = establish_connection(&args).unwrap_or_else(|e| {
        error!("Failed to connect to server: {}", e);
        std::process::exit(1);
    });
    commands.insert_resource(ServerConnection(stream));
    info!("Connected");
}

fn setup_view(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn update_from_server(
    mut commands: Commands,
    mut server: ResMut<ServerConnection>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let server_commands = read_commands(&mut server.0).unwrap_or_else(|e| {
        error!("Failed to read commands from server: {}", e);
        std::process::exit(1);
    });
    for command in server_commands {
        match command {
            Command::EstablishRole(Role::Hoop { x }) => {
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
        }
    }
}

fn establish_connection(args: &Args) -> anyhow::Result<TcpStream> {
    let stream = TcpStream::connect((args.server.as_str(), args.port))?;
    stream.set_nonblocking(true)?;
    Ok(stream)
}
