use tokio::net::UdpSocket;
use udp_playground::framing;

#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let sock = UdpSocket::bind("0.0.0.0:8080").await?;
  let mut buf = [0u8; 1024];

  loop {
    let (len, addr) = sock.recv_from(&mut buf).await?;
    if let Some(payload) = framing::decode(&buf[..len]) {
      println!("from {}: {:?}", &addr, payload);
      sock.send_to(&buf[..len], &addr).await?;
    }
  }

}
