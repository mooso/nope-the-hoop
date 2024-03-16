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
pub struct Ball {
    id: u32,
}

pub type BallQuery<'world, 'state, 'a> =
    Query<'world, 'state, (Entity, &'a Ball, &'a mut Transform)>;

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
            .add(Circle {
                radius: BALL_RADIUS,
            })
            .into();
        Self {
            ball_mesh,
            ball_material,
        }
    }
}

pub fn add_ball(commands: &mut Commands, id: u32, position: Point, asset_handles: &AssetHandles) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: asset_handles.ball_mesh.clone(),
            material: asset_handles.ball_material.clone(),
            transform: Transform::from_translation(Vec3::new(position.x, position.y, 0.)),
            ..default()
        },
        Ball { id },
    ));
}

pub fn remove_ball(commands: &mut Commands, id: u32, ball_query: &mut BallQuery) {
    let Some((entity, _, _)) = ball_query.iter().find(|(_, b, _)| b.id == id) else {
        return;
    };
    commands.entity(entity).despawn();
}

pub fn move_ball(id: u32, position: Point, ball_query: &mut BallQuery) {
    let Some((_, _, mut transform)) = ball_query.iter_mut().find(|(_, b, _)| b.id == id) else {
        return;
    };
    transform.translation.x = position.x;
    transform.translation.y = position.y;
}

fn setup_throw_angle(mut commands: Commands) {
    commands.insert_resource(ThrowAngle(0.));
}

fn handle_input(
    current_role: Res<CurrentRole>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut throw_angle: ResMut<ThrowAngle>,
) {
    let Role::Ball { .. } = current_role.0 else {
        return;
    };
    let mut factor = 0.;
    if keyboard_input.pressed(KeyCode::ArrowLeft) {
        factor += 1.;
    }
    if keyboard_input.pressed(KeyCode::ArrowRight) {
        factor -= 1.;
    }
    throw_angle.0 += GUIDE_SPEED * factor * time.delta_seconds();
}

fn draw_guide(
    mut gizmos: Gizmos,
    current_role: Res<CurrentRole>,
    throw_angle: Res<ThrowAngle>,
    ball_query: BallQuery,
) {
    let Role::Ball { id } = current_role.0 else {
        return;
    };
    let Some((_, _, transform)) = ball_query.iter().find(|(_, b, _)| b.id == id) else {
        return;
    };
    // Unit vector in the direction of the throw
    let throw_direction = Vec2::new(throw_angle.0.cos(), throw_angle.0.sin());
    let guide_start =
        transform.translation.truncate() + throw_direction * (BALL_RADIUS + GUIDE_MARGIN);
    gizmos.ray_2d(guide_start, throw_direction * GUIDE_LENGTH, Color::WHITE);
}
