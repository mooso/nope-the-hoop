use std::io::{ErrorKind, Read, Write};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Role {
    Hoop { x: f32 },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ToClientMessage {
    EstablishRole(Role),
    MoveHoop { x: f32 },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ToServerMessage {
    MoveHoop { x: f32 },
}

fn should_break(io_error_kind: ErrorKind) -> bool {
    match io_error_kind {
        ErrorKind::WouldBlock => true,
        #[cfg(test)]
        ErrorKind::UnexpectedEof => true,
        _ => false,
    }
}

fn read_messages<T: DeserializeOwned>(mut stream: impl Read) -> anyhow::Result<Vec<T>> {
    let mut commands = vec![];
    loop {
        let result = ciborium::from_reader::<T, _>(&mut stream);
        match result {
            Ok(result) => commands.push(result),
            Err(ciborium::de::Error::Io(e)) if should_break(e.kind()) => {
                break;
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(commands)
}

pub fn read_messages_as_client(mut stream: impl Read) -> anyhow::Result<Vec<ToClientMessage>> {
    read_messages(&mut stream)
}

pub fn read_messages_as_server(mut stream: impl Read) -> anyhow::Result<Vec<ToServerMessage>> {
    read_messages(&mut stream)
}

pub fn write_message(mut stream: impl Write, command: &impl Serialize) -> anyhow::Result<()> {
    ciborium::ser::into_writer(command, &mut stream)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut buf = vec![];
        let messages = vec![
            ToClientMessage::EstablishRole(Role::Hoop { x: 1.0 }),
            ToClientMessage::EstablishRole(Role::Hoop { x: 2.0 }),
        ];
        for message in &messages {
            write_message(&mut buf, message).expect("write_command");
        }
        let result = read_messages(&*buf).expect("read_commands");
        assert_eq!(messages, result);
    }
}
