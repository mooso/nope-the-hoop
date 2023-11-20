use std::io::{ErrorKind, Read, Write};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Role {
    Hoop { x: f32 },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Message {
    EstablishRole(Role),
}

fn should_break(io_error_kind: ErrorKind) -> bool {
    match io_error_kind {
        ErrorKind::WouldBlock => true,
        #[cfg(test)]
        ErrorKind::UnexpectedEof => true,
        _ => false,
    }
}

pub fn read_messages(mut stream: impl Read) -> anyhow::Result<Vec<Message>> {
    let mut commands = vec![];
    loop {
        let result = ciborium::from_reader::<Message, _>(&mut stream);
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

pub fn write_message(mut stream: impl Write, command: &Message) -> anyhow::Result<()> {
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
            Message::EstablishRole(Role::Hoop { x: 1.0 }),
            Message::EstablishRole(Role::Hoop { x: 2.0 }),
        ];
        for message in &messages {
            write_message(&mut buf, message).expect("write_command");
        }
        let result = read_messages(&*buf).expect("read_commands");
        assert_eq!(messages, result);
    }
}
