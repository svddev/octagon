use snow::Builder;
use tokio::net::UdpSocket;
use tokio::time::{sleep, Duration};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let peer: SocketAddr = std::env::args()
  .nth(1)
  .expect("peer addr required")
  .parse()?;

  let socket = Arc::new(UdpSocket::bind("0.0.0.0:8080").await?);

  let builder = Builder::new("Noise_XX_25519_ChaChaPoly_BLAKE2s".parse()?);
  let static_key = builder.generate_keypair()?;

  let is_initiator = std::env::var("INITIATOR").is_ok();

  let noise = if is_initiator {
    builder.local_private_key(&static_key.private)?.build_initiator()?
  } else {
    builder.local_private_key(&static_key.private)?.build_responder()?
  };

  let noise = Arc::new(Mutex::new(noise));
  let last_packet = Arc::new(Mutex::new(None::<Vec<u8>>));

  // Sender / retransmitter
  {
    let socket = socket.clone();
    let noise = noise.clone();
    let last_packet = last_packet.clone();

    tokio::spawn(async move {
      loop {
        let packet = {
          let mut n = noise.lock().await;

          if n.is_handshake_finished() {
            return;
          }

          if !n.is_my_turn() {
            None
          } else {
            let mut buf = vec![0u8; 2048];
            match n.write_message(&[], &mut buf) {
              Ok(len) => {
                buf.truncate(len);
                *last_packet.lock().await = Some(buf.clone());
                Some(buf)
              }
              Err(_) => None,
            }
          }
        };

        if let Some(pkt) = packet {
          let _ = socket.send_to(&pkt, peer).await;
        } else if let Some(pkt) = last_packet.lock().await.clone() {
          let _ = socket.send_to(&pkt, peer).await;
        }

        sleep(Duration::from_millis(100)).await;
      }
    });
  }

  // Receiver
  let mut buf = [0u8; 2048];
  loop {
    let (len, _) = socket.recv_from(&mut buf).await?;

    let done = {
      let mut n = noise.lock().await;
      let _ = n.read_message(&buf[..len], &mut []);
      n.is_handshake_finished()
    };

    if done {
      break;
    }
  }

  println!("🔐 Noise handshake complete");
  Ok(())
}
