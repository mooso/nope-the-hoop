use std::io::Read;

use clap::Parser;
use nope_the_hoop_proto::{
    read_messages_as_server, write_message, HorizontalDirection, ToClientMessage, ToServerMessage,
};
use tokio::{
    io::{AsyncWriteExt, Interest},
    net::{
        tcp::{ReadHalf, WriteHalf},
        TcpListener, TcpStream,
    },
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
    let _guard =
        tracing::subscriber::set_global_default(tracing_subscriber::fmt::Subscriber::new());
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

const INITIAL_HOOP_X: f32 = 100.;
const HOOP_MIN_X: f32 = 0.;
const HOOP_MAX_X: f32 = 200.;
const HOOOP_SPEED: f32 = 100.;

async fn process(stream: &mut TcpStream) -> anyhow::Result<()> {
    let (read, mut write) = stream.split();
    let mut read = ReadWrap(read);
    let mut hoop_x = INITIAL_HOOP_X;
    write_to_client(&mut write, &ToClientMessage::EstablishAsHoop { x: hoop_x }).await?;
    loop {
        read.0.ready(Interest::READABLE).await?;
        let client_messages = read_messages_as_server(&mut read)?;
        for message in client_messages {
            match message {
                ToServerMessage::MoveHoop {
                    direction,
                    seconds_pressed,
                } => {
                    let sign = match direction {
                        HorizontalDirection::Left => -1.,
                        HorizontalDirection::Right => 1.,
                    };
                    let delta_x = sign * HOOOP_SPEED * seconds_pressed;
                    hoop_x = (hoop_x + delta_x).clamp(HOOP_MIN_X, HOOP_MAX_X);
                    write_to_client(&mut write, &ToClientMessage::MoveHoop { x: hoop_x }).await?;
                }
            }
        }
    }
}

async fn write_to_client(
    write: &mut WriteHalf<'_>,
    command: &ToClientMessage,
) -> anyhow::Result<()> {
    let mut buf = vec![];
    write_message(&mut buf, command)?;
    write.write_all(&buf).await?;
    Ok(())
}
