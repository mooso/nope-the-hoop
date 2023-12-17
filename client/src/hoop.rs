use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use nope_the_hoop_proto::message::{HorizontalDirection, ToServerMessage};

use crate::{CurrentRole, HandleErrors, Role, ServerConnection};

#[derive(Component)]
pub struct Hoop;

pub type HoopQuery<'world, 'state, 'a> = Query<'world, 'state, (&'a Hoop, &'a mut Transform)>;

pub fn setup(app: &mut App) {
    app.add_systems(Update, handle_input);
}

pub struct AssetHandles {
    hoop_mesh: Mesh2dHandle,
    hoop_material: Handle<ColorMaterial>,
}

impl AssetHandles {
    pub fn create(materials: &mut Assets<ColorMaterial>, meshes: &mut Assets<Mesh>) -> Self {
        let hoop_material = materials.add(ColorMaterial::from(Color::GRAY));
        let hoop_mesh = meshes
            .add(shape::Quad::new(Vec2::new(50., 10.)).into())
            .into();
        Self {
            hoop_mesh,
            hoop_material,
        }
    }
}

pub fn add_hoop(commands: &mut Commands, hoop_x: f32, asset_handles: &AssetHandles) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: asset_handles.hoop_mesh.clone(),
            material: asset_handles.hoop_material.clone(),
            transform: Transform::from_translation(Vec3::new(hoop_x, 0., 0.)),
            ..default()
        },
        Hoop,
    ));
}

pub fn move_hoop(hoops: &mut HoopQuery, x: f32) {
    hoops.single_mut().1.translation.x = x;
}

fn handle_input(
    mut server: ResMut<ServerConnection>,
    current_role: Res<CurrentRole>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let Role::Hoop = current_role.0 else {
        return;
    };
    if keyboard_input.pressed(KeyCode::Left) {
        send_hoop_movement(&mut server, HorizontalDirection::Left, &time);
    }
    if keyboard_input.pressed(KeyCode::Right) {
        send_hoop_movement(&mut server, HorizontalDirection::Right, &time);
    }
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
