use tokio::net::UdpSocket;
use udp_playground::framing;

#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let sock = UdpSocket::bind("0.0.0.0:0").await?;
  let msg = framing::encode(b"hello");

  sock.send_to(&msg, "127.0.0.1:8080").await?;

  let mut buf = [0u8; 2048];
  let (len, _) = sock.recv_from(&mut buf).await?;
  println!("reply: {:?}", &buf[..len]);

  Ok(())
}
