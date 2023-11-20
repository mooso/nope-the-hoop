use clap::Parser;
use tokio::net::TcpListener;
use tracing::info;

#[derive(Parser)]
#[command(
    author = "Mostafa",
    version = "0",
    about = "Server for the hit nope-the-hoop game"
)]
struct Args {
    /// The port to bind to.
    #[arg(short, long, default_value_t = 0)]
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
        let (_socket, addr) = listener.accept().await.unwrap();
        // A new task is spawned for each inbound socket. The socket is
        // moved to the new task and processed there.
        tokio::spawn(async move {
            info!("Accepted connection from {}", addr);
        });
    }
}
