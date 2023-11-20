use std::io::Read;

use clap::Parser;
use nope_the_hoop_proto::{read_messages, write_message, Message, Role};
use tokio::{
    io::AsyncWriteExt,
    net::{tcp::ReadHalf, TcpListener, TcpStream},
};
use tracing::info;

#[derive(Parser)]
#[command(
    author = "Mostafa",
    version = "0",
    about = "Server for the hit nope-the-hoop game"
)]
struct Args {
    /// The port to bind to.
    #[arg(short, long, default_value_t = 7434)]
    port: u16,

    /// The address to bind to.
    #[arg(long, default_value = "127.0.0.1")]
    bind_address: String,
}

#[tokio::main]
async fn main() {
    let _guard = tracing::subscriber::set_default(tracing_subscriber::fmt::Subscriber::new());
    let args = Args::parse();
    let listener = TcpListener::bind(&format!("{}:{}", args.bind_address, args.port))
        .await
        .unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());

    loop {
        let (mut stream, addr) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            info!("Accepted connection from {}", addr);
            match process(&mut stream).await {
                Ok(_) => info!("Connection from {} ended successfully", addr),
                Err(e) => info!("Connection from {} failed: {}", addr, e),
            }
        });
    }
}

struct ReadWrap<'a>(ReadHalf<'a>);

impl Read for ReadWrap<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.try_read(buf)
    }
}

async fn process(stream: &mut TcpStream) -> anyhow::Result<()> {
    let (read, mut write) = stream.split();
    let mut read = ReadWrap(read);
    let mut buf = vec![];
    write_message(&mut buf, &Message::EstablishRole(Role::Hoop { x: 100. }))?;
    write.write(&buf).await?;
    loop {
        let _client_messages = read_messages(&mut read);
    }
}
