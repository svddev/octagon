use tokio::net::UdpSocket;


#[allow(unreachable_code)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let socket = UdpSocket::bind("0.0.0.0:9876").await?;
  let mut buf = [0u8; 1024];
  loop {
    let (_len, addr) = socket.recv_from(&mut buf).await?;
    let response = format!("{}", addr);
    socket.send_to(response.as_bytes(), addr).await?;
  }
}
