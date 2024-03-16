use serde::{Deserialize, Serialize};

use crate::state::{self};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum HorizontalDirection {
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ToClientMessage {
    InitialState(state::GameState),
    EstablishAsHoop,
    EstablishAsBall { id: u32 },
    EstablishAsObserver,
    UpdateState(state::UpdateState),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ToServerMessage {
    Hello {
        game_id: u32,
    },
    MoveHoop {
        direction: HorizontalDirection,
        seconds_pressed: f32,
    },
    ShootBall {
        id: u32,
        angle: f32,
        seconds_pressed: f32,
    },
}
