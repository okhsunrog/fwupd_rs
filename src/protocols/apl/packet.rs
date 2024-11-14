#[repr(C, packed)]
pub struct AplHeader {
    pub type_id: u8,  // 3 bits type, 5 bits id
}

#[repr(C, packed)]
pub struct AplDataPacket {
    pub header: AplHeader,
    pub block_number: u16,
    // Variable length data follows
}

#[repr(C, packed)]
pub struct AplAckPacket {
    pub header: AplHeader,
    pub block_number: u16,
}

#[repr(C, packed)]
pub struct AplErrorPacket {
    pub header: AplHeader,
    pub block_number: u16,
    pub error_code: u8,
    // Variable length error message follows
}

#[repr(C, packed)]
pub struct AplRequestPacket {
    pub header: AplHeader,
    pub block_size: u16,
    pub timeout: u16,
    pub command: u8,
    pub offset: u32,
    pub length: u32,
}

impl AplDataPacket {
    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(3 + self.data.len());
        buf.put_u8(self.header.type_id);
        buf.put_u16_le(self.block_number);
        buf.extend_from_slice(&self.data);
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 3 {
            return Err(Error::new(ErrorKind::InvalidData, "Packet too short"));
        }
        Ok(Self {
            header: AplHeader { type_id: bytes[0] },
            block_number: u16::from_le_bytes([bytes[1], bytes[2]]),
            data: bytes[3..].to_vec(),
        })
    }
}

impl AplAckPacket {
    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(3);
        buf.put_u8(self.header.type_id);
        buf.put_u16_le(self.block_number);
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 3 {
            return Err(Error::new(ErrorKind::InvalidData, "Packet too short"));
        }
        Ok(Self {
            header: AplHeader { type_id: bytes[0] },
            block_number: u16::from_le_bytes([bytes[1], bytes[2]]),
        })
    }
}

impl AplErrorPacket {
    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(4 + self.error_message.len());
        buf.put_u8(self.header.type_id);
        buf.put_u16_le(self.block_number);
        buf.put_u8(self.error_code);
        buf.extend_from_slice(self.error_message.as_bytes());
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 4 {
            return Err(Error::new(ErrorKind::InvalidData, "Packet too short"));
        }
        Ok(Self {
            header: AplHeader { type_id: bytes[0] },
            block_number: u16::from_le_bytes([bytes[1], bytes[2]]),
            error_code: bytes[3],
            error_message: String::from_utf8_lossy(&bytes[4..]).to_string(),
        })
    }
}

impl AplRequestPacket {
    pub fn to_bytes(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(12);
        buf.put_u8(self.header.type_id);
        buf.put_u16_le(self.block_size);
        buf.put_u16_le(self.timeout);
        buf.put_u8(self.command);
        buf.put_u32_le(self.offset);
        buf.put_u32_le(self.length);
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < 12 {
            return Err(Error::new(ErrorKind::InvalidData, "Packet too short"));
        }
        Ok(Self {
            header: AplHeader { type_id: bytes[0] },
            block_size: u16::from_le_bytes([bytes[1], bytes[2]]),
            timeout: u16::from_le_bytes([bytes[3], bytes[4]]),
            command: bytes[5],
            offset: u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]),
            length: u32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]),
        })
    }
}
