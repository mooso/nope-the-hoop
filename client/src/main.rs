mod ball;
mod connection;
mod hoop;

use std::fmt::Display;

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

enum Role {
    Unknown,
    Hoop,
    Ball { origin: Vec2 },
}

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
        .add_systems(Startup, (setup_view, setup_role, setup_assets));
    connection::setup(&mut app);
    ball::setup(&mut app);
    hoop::setup(&mut app);
    app.run();
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
