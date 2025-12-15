use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use snow::Builder;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::sleep;


#[allow(unused_mut)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let peer: SocketAddr = std::env::args()
    .nth(1)
    .expect("Peer address should be provided")
    .parse()?;
  println!("Peer  addr: {}", peer);

  let socket = Arc::new(UdpSocket::bind("0.0.0.0:8080").await?);

  let stun_address = stun(socket.clone()).await?;
  println!("Stun result: {}", stun_address);

  let builder = Builder::new("Noise_XX_25519_ChaChaPoly_BLAKE2s".parse()?);
  let static_key = builder.generate_keypair()?;

  let is_initiator = std::env::var("INITIATOR").is_ok();
  let mut noise = if is_initiator {
    println!("Building INITIATOR");
    builder
      .local_private_key(&static_key.private)?
      .build_initiator()?
  } else {
    println!("Building RESPONDER");
    builder
      .local_private_key(&static_key.private)?
      .build_responder()?
  };


  let noise = Arc::new(Mutex::new(noise));

  let sender = {
    let socket = socket.clone();
    let noise = noise.clone();
    tokio::spawn(async move {
      let mut buf = [0u8; 2048];
      loop {
        let mut n = noise.lock().await;
        if n.is_handshake_finished() {
          break;
        }

        if let Ok(len) = n.write_message(&[], &mut buf) {
          let _ = socket.send_to(&buf[..len], peer).await;
        }
        drop(n);

        sleep(Duration::from_millis(100)).await;
      }
    })
  };

  let mut buf = [0u8; 2048];

  loop {
    let (len, _) = socket.recv_from(&mut buf).await?;
    let mut n = noise.lock().await;
    let _ = n.read_message(&buf[..len], &mut []);

    if n.is_handshake_finished() {
      break;
    }
  }

  sender.abort();

  println!("Session keys established");

  Ok(())

}

async fn stun(socket: Arc<UdpSocket>) -> anyhow::Result<String> {
  socket.send_to(b"Hello There", "72.56.90.50:9876").await?;
  let mut buf = [0u8; 1024];
  let (len, _) = socket.recv_from(&mut buf).await?;
  Ok(std::str::from_utf8(&buf[..len])?.to_string())
}
