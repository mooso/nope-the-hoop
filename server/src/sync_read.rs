use std::io::Read;

use tokio::net::tcp::OwnedReadHalf;

pub struct ReadWrap(OwnedReadHalf);

pub fn wrap(read_half: OwnedReadHalf) -> ReadWrap {
    ReadWrap(read_half)
}

impl ReadWrap {
    pub async fn await_ready(&self) -> tokio::io::Result<()> {
        self.0.readable().await
    }
}

impl Read for ReadWrap {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.try_read(buf)
    }
}
