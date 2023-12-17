use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use nope_the_hoop_proto::state::Point;

use crate::{CurrentRole, Role};

const BALL_RADIUS: f32 = 10.;
const GUIDE_MARGIN: f32 = 1.;
const GUIDE_LENGTH: f32 = 20.;
const GUIDE_SPEED: f32 = 10.;

#[derive(Component)]
struct Ball;

#[derive(Resource)]
struct ThrowAngle(f32);

pub fn setup(app: &mut App) {
    app.add_systems(Startup, setup_throw_angle)
        .add_systems(Update, (handle_input, draw_guide));
}

pub struct AssetHandles {
    ball_mesh: Mesh2dHandle,
    ball_material: Handle<ColorMaterial>,
}

impl AssetHandles {
    pub fn create(materials: &mut Assets<ColorMaterial>, meshes: &mut Assets<Mesh>) -> Self {
        let ball_material = materials.add(ColorMaterial::from(Color::RED));
        let ball_mesh = meshes
            .add(
                shape::Circle {
                    radius: BALL_RADIUS,
                    ..default()
                }
                .into(),
            )
            .into();
        Self {
            ball_mesh,
            ball_material,
        }
    }
}

pub fn add_ball(commands: &mut Commands, position: Point, asset_handles: &AssetHandles) {
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

fn setup_throw_angle(mut commands: Commands) {
    commands.insert_resource(ThrowAngle(0.));
}

fn handle_input(
    current_role: Res<CurrentRole>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut throw_angle: ResMut<ThrowAngle>,
) {
    let Role::Ball { .. } = current_role.0 else {
        return;
    };
    let mut factor = 0.;
    if keyboard_input.pressed(KeyCode::Left) {
        factor += 1.;
    }
    if keyboard_input.pressed(KeyCode::Right) {
        factor -= 1.;
    }
    throw_angle.0 += GUIDE_SPEED * factor * time.delta_seconds();
}

fn draw_guide(mut gizmos: Gizmos, current_role: Res<CurrentRole>, throw_angle: Res<ThrowAngle>) {
    let Role::Ball { origin } = current_role.0 else {
        return;
    };
    // Unit vector in the direction of the throw
    let throw_direction = Vec2::new(throw_angle.0.cos(), throw_angle.0.sin());
    let guide_start = origin + throw_direction * (BALL_RADIUS + GUIDE_MARGIN);
    gizmos.ray_2d(guide_start, throw_direction * GUIDE_LENGTH, Color::WHITE);
}
