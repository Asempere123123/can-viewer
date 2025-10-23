use futures_util::SinkExt;
use socketcan::{EmbeddedFrame, Frame, tokio::CanSocket};
use std::fmt;
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{Receiver, channel};
use tokio_tungstenite::tungstenite::Message;

struct Hex<'a>(&'a [u8]);

impl fmt::UpperHex for Hex<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &b in self.0 {
            write!(f, "{:02X}", b)?;
        }
        Ok(())
    }
}

async fn handle_conn(stream: TcpStream, mut msg_recv: Receiver<Arc<str>>) {
    let Ok(mut ws) = tokio_tungstenite::accept_async(stream).await else {
        return;
    };

    loop {
        let Ok(msg) = msg_recv.recv().await else {
            continue;
        };

        if ws.send(Message::text(&*msg)).await.is_err() {
            return;
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    let _ = dotenvy::dotenv().map_err(|e| {
        if !e.not_found() {
            println!("Failed to read dotenv file {:?}", e);
        }
        e
    });

    let can_socket =
        CanSocket::open(&std::env::var("CAN_SOCKET").expect("CAN_SOCKET env var must be set"))?;
    let tcp_listener =
        TcpListener::bind(std::env::var("HOST_ADDR").expect("HOST_ADDR env var must be set"))
            .await?;

    let start_time = SystemTime::now();
    let time = Instant::now();

    let (message_sender, message_recv) = channel(128);

    // Accept task
    tokio::spawn(async move {
        loop {
            let Ok((tcp_stream, _addr)) = tcp_listener.accept().await else {
                continue;
            };

            tokio::spawn(handle_conn(tcp_stream, message_recv.resubscribe()));
        }
    });

    loop {
        let Ok(frame) = can_socket.read_frame().await else {
            continue;
        };

        let id = frame.raw_id();
        let data = frame.data();

        let timestamp = start_time + time.elapsed();
        // UNSAFE: Time goes forward
        let timestamp = unsafe { timestamp.duration_since(UNIX_EPOCH).unwrap_unchecked() };

        let _ = message_sender.send(Arc::from(format!(
            "({}.{}) can0 {:X}#{:X}",
            timestamp.as_secs(),
            timestamp.subsec_nanos(),
            id,
            Hex(data)
        )));
    }
}
