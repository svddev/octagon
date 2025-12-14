use std::{net::SocketAddr, time::Duration};
use std::sync::Arc;
use tokio::{net::UdpSocket, time::sleep};


#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let peer_addr: SocketAddr = std::env::args()
    .nth(1)
    .expect("peer addr required")
    .parse()?;

  let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
  socket.connect(peer_addr).await?;

  println!("Punching to {}", peer_addr);

  let sender = {
    let socket = socket.clone();
    tokio::spawn(async move {
      loop {
        let _ = &socket.send(b"punch").await;
        sleep(Duration::from_millis(200)).await;
      }
    })
  };

  let mut buf = [0u8; 1024];

  loop {
    let len = socket.recv(&mut buf).await?;
    println!("CONNECTED! received {} bytes: {:?}", len, &buf[..len]);
    break;
  }

  sender.abort();

  Ok(())
}
