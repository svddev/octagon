use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};
use std::{net::SocketAddr, sync::Arc};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let peer: SocketAddr = std::env::args()
  .nth(1)
  .expect("peer addr required")
  .parse()?;

  let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);

  socket.send_to(b"Hello There", "72.56.90.50:9876").await?;
  let mut buf = [0u8; 1024];
  let (len, _) = socket.recv_from(&mut buf).await?;
  let public_addr = std::str::from_utf8(&buf[..len]).unwrap();
  println!("{}", public_addr);


  println!("Local addr: {}", socket.local_addr()?);
  println!("Peer  addr: {}", peer);

  //let received = Arc::new(tokio::sync::Notify::new());
  //let received_clone = received.clone();

  // Sender
  let sender = {
    let socket = socket.clone();
    tokio::spawn(async move {
      loop {
        let _ = socket.send_to(b"PING", peer).await;
        sleep(Duration::from_millis(100)).await;
      }
    })
  };

  let mut buf = [0u8; 1024];
  let mut got_ping = false;
  let mut got_pong = false;

  loop {
    let (len, from) = socket.recv_from(&mut buf).await?;
    let msg = &buf[..len];

    if msg == b"PING" {
      got_ping = true;
      socket.send_to(b"PONG", from).await?;
    } else if msg == b"PONG" {
      got_pong = true;
    }

    if got_ping && got_pong {
      println!("FULLY CONNECTED with {}", from);
      break;
    }
  }

  sender.abort();
  Ok(())
}
