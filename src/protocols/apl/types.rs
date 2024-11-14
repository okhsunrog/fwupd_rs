use bytes::BytesMut;
use std::io::{Error, ErrorKind};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AplRequestType {
    None = 0,
    ReadRequest = 1,
    WriteRequest = 2,
    Data = 3,
    Ack = 4,
    Error = 5,
}

impl TryFrom<u8> for AplRequestType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::ReadRequest),
            2 => Ok(Self::WriteRequest),
            3 => Ok(Self::Data),
            4 => Ok(Self::Ack),
            5 => Ok(Self::Error),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Invalid packet type: {}", value)
            )),
        }
    }
}


#[derive(Debug)]
pub struct AplMessage {
    pub packet_type: AplRequestType,
    pub block_number: u16,
    pub data: Vec<u8>,
}

impl AplMessage {
    pub fn new(packet_type: AplRequestType, block_number: u16, data: Vec<u8>) -> Self {
        Self {
            packet_type,
            block_number,
            data,
        }
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, Error> {
        if data.len() < 3 {
            return Err(Error::new(ErrorKind::InvalidData, "Message too short"));
        }

        let packet_type = AplRequestType::try_from(data[0])?;
        let block_number = u16::from_le_bytes([data[1], data[2]]);
        let data = data[3..].to_vec();

        Ok(Self {
            packet_type,
            block_number,
            data,
        })
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(3 + self.data.len());
        buf.extend_from_slice(&[self.packet_type as u8]);
        buf.extend_from_slice(&self.block_number.to_le_bytes());
        buf.extend_from_slice(&self.data);
        buf
    }
}
