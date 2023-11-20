use std::net::TcpStream;

use bevy::prelude::*;
use clap::Parser;

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

#[derive(Resource)]
struct ServerConnection(TcpStream);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup_connect, setup_view))
        .run();
}

fn setup_connect(mut commands: Commands) {
    let args = Args::parse();
    info!("Connecting to {}:{}", args.server, args.port);
    let stream = TcpStream::connect((args.server.as_str(), args.port)).unwrap_or_else(|e| {
        error!("Failed to connect to server: {}", e);
        std::process::exit(1);
    });
    commands.insert_resource(ServerConnection(stream));
    info!("Connected");
}

fn setup_view(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
