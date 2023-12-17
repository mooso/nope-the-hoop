use serde::{Deserialize, Serialize};

use crate::state::{self, Point};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum HorizontalDirection {
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ToClientMessage {
    InitialState(state::GameState),
    EstablishAsHoop,
    EstablishAsBall { origin: Point },
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
}
