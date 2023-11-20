use std::io::{ErrorKind, Read, Write};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Role {
    Hoop { x: f32 },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Command {
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

pub fn read_commands(mut stream: impl Read) -> anyhow::Result<Vec<Command>> {
    let mut commands = vec![];
    loop {
        let result = ciborium::from_reader::<Command, _>(&mut stream);
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

pub fn write_command(mut stream: impl Write, command: &Command) -> anyhow::Result<()> {
    ciborium::ser::into_writer(command, &mut stream)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let mut buf = vec![];
        let commands = vec![
            Command::EstablishRole(Role::Hoop { x: 1.0 }),
            Command::EstablishRole(Role::Hoop { x: 2.0 }),
        ];
        for command in &commands {
            write_command(&mut buf, command).expect("write_command");
        }
        let result = read_commands(&*buf).expect("read_commands");
        assert_eq!(commands, result);
    }
}
