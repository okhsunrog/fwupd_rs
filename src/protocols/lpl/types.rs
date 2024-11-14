use bytes::BytesMut;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub struct LplMessage {
    pub syn: bool,
    pub payload: Vec<u8>,
    pub crc: u16,
}

impl LplMessage {
    pub fn new(payload: Vec<u8>, crc: u16) -> Self {
        Self {
            syn: false,
            payload,
            crc,
        }
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, Error> {
        if data.is_empty() {
            return Err(Error::new(ErrorKind::InvalidData, "Empty message"));
        }

        let syn = data[0] == 0x55;
        let payload = data[1..data.len()-2].to_vec();
        let crc = u16::from_le_bytes([
            data[data.len()-2],
            data[data.len()-1]
        ]);

        Ok(Self { syn, payload, crc })
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(self.payload.len() + 3);
        if self.syn {
            buf.extend_from_slice(&[0x55]);
        }
        buf.extend_from_slice(&self.payload);
        buf.extend_from_slice(&self.crc.to_le_bytes());
        buf
    }
}
