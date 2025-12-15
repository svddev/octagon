use std::net::SocketAddr;
use std::sync::Arc;
use snow::Builder;
use tokio::net::UdpSocket;



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


  let mut buf = [0u8; 1024];
  let mut msg = [0u8; 1024];
  loop {
    if noise.is_handshake_finished() {
      println!("🔐 Noise handshake complete");
      break;
    }

    if noise.is_my_turn() {
      let len = noise.write_message(&[], &mut msg)?;
      socket.send_to(&msg[..len], peer).await?;
    }

    let (len, _) = socket.recv_from(&mut buf).await?;
    noise.read_message(&buf[..len], &mut [])?;
  }

  println!("Session keys established");

  Ok(())

}

async fn stun(socket: Arc<UdpSocket>) -> anyhow::Result<String> {
  socket.send_to(b"Hello There", "72.56.90.50:9876").await?;
  let mut buf = [0u8; 1024];
  let (len, _) = socket.recv_from(&mut buf).await?;
  Ok(std::str::from_utf8(&buf[..len])?.to_string())
}
