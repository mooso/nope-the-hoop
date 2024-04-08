use std::{collections::HashMap, time::Duration};

use nope_the_hoop_proto::{
    message::{HorizontalDirection, ToClientMessage},
    state::{GameState, Point, UpdateState},
};

const INITIAL_HOOP_X: f32 = 100.;
const HOOP_MIN_X: f32 = 0.;
const HOOP_MAX_X: f32 = 200.;
const HOOOP_SPEED: f32 = 100.;
const SINGLE_BALL_POSITION: Point = Point { x: -100., y: 10. };
const BALL_SPEED_PER_SECOND_PRESSED: f32 = 100.;
const BALL_MAX_SPEED: f32 = 100.;
const GRAVITY: f32 = 9.81;

pub(crate) struct Game {
    state: GameState,
    ball_velocities: HashMap<u32, Option<Point>>,
}

impl Default for Game {
    fn default() -> Self {
        let mut ball_positions = HashMap::new();
        let mut ball_velocities = HashMap::new();
        ball_positions.insert(0, SINGLE_BALL_POSITION);
        ball_velocities.insert(0, None);
        Self {
            state: GameState {
                hoop_x: INITIAL_HOOP_X,
                ball_positions,
            },
            ball_velocities,
        }
    }
}

impl Game {
    pub(crate) fn state(&self) -> &GameState {
        &self.state
    }

    pub(crate) fn move_hoop(&mut self, direction: HorizontalDirection, seconds_pressed: f32) {
        let sign = match direction {
            HorizontalDirection::Left => -1.,
            HorizontalDirection::Right => 1.,
        };
        let delta_x = sign * HOOOP_SPEED * seconds_pressed;
        self.state.hoop_x = (self.state.hoop_x + delta_x).clamp(HOOP_MIN_X, HOOP_MAX_X);
    }

    pub(crate) fn shoot_ball(&mut self, id: u32, angle: f32, seconds_pressed: f32) {
        let ball_velocity = calculate_ball_velocity(angle, seconds_pressed);
        self.ball_velocities.insert(id, Some(ball_velocity));
    }

    pub(crate) fn update(&mut self, elapsed: Duration, updates: &mut Vec<ToClientMessage>) {
        for (id, velocity) in self.ball_velocities.iter_mut() {
            let Some(velocity) = velocity else {
                continue;
            };
            let Some(ball) = self.state.ball_positions.get_mut(id) else {
                continue;
            };
            ball.x += velocity.x * elapsed.as_secs_f32();
            ball.y += velocity.y * elapsed.as_secs_f32();
            updates.push(ToClientMessage::UpdateState(UpdateState::MoveBall {
                id: *id,
                position: *ball,
            }));
            velocity.y -= GRAVITY * elapsed.as_secs_f32();
        }
    }
}

fn calculate_ball_velocity(angle: f32, seconds_pressed: f32) -> Point {
    let speed = (seconds_pressed * BALL_SPEED_PER_SECOND_PRESSED).clamp(0., BALL_MAX_SPEED);
    let x = angle.cos() * speed;
    let y = angle.sin() * speed;
    Point { x, y }
}
