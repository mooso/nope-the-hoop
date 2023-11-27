use std::{pin::Pin, task::Poll};

use anyhow::Context;
use futures::Stream;
use pin_project::pin_project;
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncRead, AsyncWrite, AsyncWriteExt, ReadBuf};

use crate::{LenType, MAX_MESSAGE_SIZE};

#[pin_project]
pub struct MessageStream<R, T> {
    #[pin]
    read: R,
    read_buf: Option<Vec<u8>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<R: AsyncRead, T: DeserializeOwned> MessageStream<R, T> {
    pub fn new(read: R) -> Self {
        Self {
            read,
            read_buf: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<R: AsyncRead, T: DeserializeOwned> Stream for MessageStream<R, T> {
    type Item = anyhow::Result<T>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        {
            let this = self.as_mut().project();
            if this.read_buf.is_none() {
                let mut len_buf = [0u8; std::mem::size_of::<LenType>()];
                match this.read.poll_read(cx, &mut ReadBuf::new(&mut len_buf)) {
                    Poll::Ready(Ok(())) => {
                        let len = u16::from_le_bytes(len_buf) as usize;
                        if len > MAX_MESSAGE_SIZE {
                            return Poll::Ready(Some(Err(anyhow::anyhow!(
                                "Message too long: {}",
                                len
                            ))));
                        }
                        *this.read_buf = Some(vec![0; len]);
                    }
                    Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(e.into()))),
                    Poll::Pending => return Poll::Pending,
                }
            }
        }
        {
            let this = self.as_mut().project();
            let read_buf = this.read_buf.as_mut().unwrap();
            match this
                .read
                .poll_read(cx, &mut ReadBuf::new(&mut read_buf[..]))
            {
                Poll::Ready(Ok(())) => {
                    let message = match ciborium::from_reader::<T, _>(&read_buf[..]) {
                        Ok(message) => message,
                        Err(e) => return Poll::Ready(Some(Err(e.into()))),
                    };
                    *this.read_buf = None;
                    Poll::Ready(Some(Ok(message)))
                }
                Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e.into()))),
                Poll::Pending => Poll::Pending,
            }
        }
    }
}

pub async fn write_message(
    stream: &mut (impl AsyncWrite + Unpin),
    messsage: &impl Serialize,
) -> anyhow::Result<()> {
    let mut buf = vec![];
    ciborium::ser::into_writer(messsage, &mut buf).context("Failed to serialize message")?;
    let len = LenType::try_from(buf.len()).context("Message too long")?;
    stream
        .write_all(&len.to_le_bytes())
        .await
        .context("Failed to write message length")?;
    stream
        .write_all(&buf)
        .await
        .context("Failed to write message")?;
    stream.flush().await.context("Failed to flush stream")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use tokio::net::{TcpListener, TcpStream};

    use super::*;

    #[tokio::test]
    async fn roundtrip() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let messages = vec!["hello".to_owned(), "x".repeat(MAX_MESSAGE_SIZE - 3)];
        let server_copy = messages.clone();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (_read, mut write) = stream.into_split();
            for message in &server_copy {
                write_message(&mut write, message).await.expect("write");
            }
        });
        let stream = TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .unwrap();
        let (read, _write) = stream.into_split();
        let mut stream = MessageStream::<_, String>::new(read);
        let mut read_messages = vec![];
        while read_messages.len() < messages.len() {
            let Some(result) = stream.next().await else {
                panic!("Stream ended before all messages were read");
            };
            read_messages.push(result.expect("read"));
        }
        assert_eq!(messages, read_messages);
        server.await.unwrap();
    }
}
