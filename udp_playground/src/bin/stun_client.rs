use tokio::net::UdpSocket;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let sock = UdpSocket::bind("0.0.0.0:0").await?;

  sock.send_to(b"Hello There", "svddev.ru:9876").await?;

  let mut buf = [0u8; 1024];
  let (len, _) = sock.recv_from(&mut buf).await?;

  let public_addr = std::str::from_utf8(&buf[..len]).unwrap();
  println!("{}", public_addr);

  Ok(())
}
