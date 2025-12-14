use bytes::{BufMut, BytesMut};

pub fn encode(payload: &[u8]) -> BytesMut {
  let mut buf = BytesMut::with_capacity(2 + payload.len());
  buf.put_u16(payload.len() as u16);
  buf.extend_from_slice(payload);
  buf
}

pub fn decode(buf: &[u8]) -> Option<&[u8]> {
  if buf.len() < 2 {
    return None;
  }
  let len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
  if buf.len() < 2 + len {
    return None;
  }
  Some(&buf[2..2 + len])
}
