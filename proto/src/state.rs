use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct GameState {
    pub hoop_x: f32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum UpdateState {
    MoveHoop { x: f32 },
}

impl UpdateState {
    pub fn apply(&self, state: &mut GameState) {
        match self {
            UpdateState::MoveHoop { x } => {
                state.hoop_x = *x;
            }
        }
    }
}
