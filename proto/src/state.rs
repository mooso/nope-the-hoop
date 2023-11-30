use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct GameState {
    pub hoop_x: f32,
    pub ball_positions: Vec<Point>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum UpdateState {
    MoveHoop { x: f32 },
    AddBall { position: Point },
}

impl UpdateState {
    pub fn apply(&self, state: &mut GameState) {
        match self {
            UpdateState::MoveHoop { x } => {
                state.hoop_x = *x;
            }
            UpdateState::AddBall { position } => {
                state.ball_positions.push(*position);
            }
        }
    }
}
