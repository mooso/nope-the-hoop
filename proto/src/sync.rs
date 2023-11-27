use anyhow::Context;
use serde::{de::DeserializeOwned, Serialize};
use std::io::{ErrorKind, Read, Write};

use crate::{LenType, MAX_MESSAGE_SIZE};

pub struct MessageStream<S> {
    stream: S,
    read_buf: Option<Vec<u8>>,
}

impl<S: Read + Write> MessageStream<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            read_buf: None,
        }
    }

    pub fn read_messages<T: DeserializeOwned>(&mut self) -> anyhow::Result<Vec<T>> {
        let mut messages = vec![];
        loop {
            if self.read_buf.is_none() {
                let mut len_buf = [0u8; std::mem::size_of::<LenType>()];
                match self.stream.read_exact(&mut len_buf) {
                    Ok(()) => (),
                    Err(e) if should_break(e.kind()) => return Ok(messages),
                    Err(e) => return Err(e.into()),
                }
                let len = u16::from_le_bytes(len_buf) as usize;
                if len > MAX_MESSAGE_SIZE {
                    return Err(anyhow::anyhow!("Message too long: {}", len));
                }
                self.read_buf = Some(vec![0; len]);
            };
            let result = self
                .stream
                .read_exact(&mut self.read_buf.as_mut().unwrap()[..]);
            match result {
                Ok(()) => {
                    let message =
                        ciborium::from_reader::<T, _>(&self.read_buf.as_ref().unwrap()[..])?;
                    self.read_buf = None;
                    messages.push(message);
                }
                Err(e) if should_break(e.kind()) => break,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(messages)
    }

    pub fn write_message(&mut self, command: &impl Serialize) -> anyhow::Result<()> {
        let mut buf = vec![];
        ciborium::ser::into_writer(command, &mut buf).context("Failed to serialize message")?;
        let len = LenType::try_from(buf.len()).context("Message too long")?;
        self.stream
            .write_all(&len.to_le_bytes())
            .context("Failed to write message length")?;
        self.stream
            .write_all(&buf)
            .context("Failed to write message")?;
        self.stream.flush().context("Failed to flush stream")?;
        Ok(())
    }
}

fn should_break(io_error_kind: ErrorKind) -> bool {
    match io_error_kind {
        ErrorKind::WouldBlock => true,
        #[cfg(test)]
        ErrorKind::UnexpectedEof => true,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::net::{TcpListener, TcpStream};

    use super::*;

    #[test]
    fn roundtrip() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let messages = vec!["hello".to_owned(), "x".repeat(MAX_MESSAGE_SIZE - 3)];
        let server_copy = messages.clone();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut stream = MessageStream::new(&mut stream);
            for message in &server_copy {
                stream.write_message(message).expect("write");
            }
        });
        let stream = TcpStream::connect(format!("127.0.0.1:{port}")).unwrap();
        stream.set_nodelay(true).unwrap();
        let mut stream = MessageStream::new(stream);
        let mut read_messages = vec![];
        while read_messages.len() < messages.len() {
            read_messages.extend(stream.read_messages::<String>().expect("read"));
        }
        assert_eq!(messages, read_messages);
        server.join().unwrap();
    }
}
