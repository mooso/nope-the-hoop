use std::io::{ErrorKind, Read, Write};

use anyhow::Context;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum HorizontalDirection {
    Left,
    Right,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ToClientMessage {
    EstablishAsHoop { x: f32 },
    MoveHoop { x: f32 },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ToServerMessage {
    MoveHoop {
        direction: HorizontalDirection,
        seconds_pressed: f32,
    },
}

fn should_break(io_error_kind: ErrorKind) -> bool {
    match io_error_kind {
        ErrorKind::WouldBlock => true,
        #[cfg(test)]
        ErrorKind::UnexpectedEof => true,
        _ => false,
    }
}

const FRAME_SIZE: usize = 64;

fn read_messages<T: DeserializeOwned>(mut stream: impl Read) -> anyhow::Result<Vec<T>> {
    let mut buf = [0; FRAME_SIZE];
    let mut commands = vec![];
    loop {
        let result = stream.read_exact(&mut buf);
        match result {
            Ok(()) => {
                let command = ciborium::from_reader::<T, _>(&buf[..])?;
                commands.push(command);
            }
            Err(e) if should_break(e.kind()) => {
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(commands)
}

pub fn read_messages_as_client(mut stream: impl Read) -> anyhow::Result<Vec<ToClientMessage>> {
    read_messages(&mut stream).context("Failed to read messages from server")
}

pub fn read_messages_as_server(mut stream: impl Read) -> anyhow::Result<Vec<ToServerMessage>> {
    read_messages(&mut stream).context("Failed to read messages from client")
}

pub fn write_message(mut stream: impl Write, command: &impl Serialize) -> anyhow::Result<()> {
    let mut buf = [0_u8; FRAME_SIZE];
    ciborium::ser::into_writer(command, &mut buf[..]).context("Failed to serialize message")?;
    stream.write_all(&buf).context("Failed to write message")?;
    stream.flush().context("Failed to flush stream")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut buf = vec![];
        let messages = vec![
            ToClientMessage::EstablishAsHoop { x: 1.0 },
            ToClientMessage::EstablishAsHoop { x: 2.0 },
        ];
        for message in &messages {
            write_message(&mut buf, message).expect("write_command");
        }
        let result = read_messages(&*buf).expect("read_commands");
        assert_eq!(messages, result);
    }
}
