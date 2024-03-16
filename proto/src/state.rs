use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct GameState {
    pub hoop_x: f32,
    pub ball_positions: HashMap<u32, Point>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum UpdateState {
    MoveHoop { x: f32 },
    AddBall { id: u32, position: Point },
    MoveBall { id: u32, position: Point },
    RemoveBall { id: u32 },
}

impl UpdateState {
    pub fn apply(&self, state: &mut GameState) {
        match self {
            UpdateState::MoveHoop { x } => {
                state.hoop_x = *x;
            }
            UpdateState::AddBall { id, position } => {
                let _previous = state.ball_positions.insert(*id, *position);
            }
            UpdateState::MoveBall { id, position } => {
                let _previous = state.ball_positions.insert(*id, *position);
            }
            UpdateState::RemoveBall { id } => {
                let _previous = state.ball_positions.remove(id);
            }
        }
    }
}
