use tokio::sync::mpsc;
use tokio_stream::{Stream, StreamExt};
use bytes::{Buf, BufMut, BytesMut};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io::{Error, ErrorKind};

mod types;
mod packet;

pub use self::types::{AplMessage, AplRequestType};
pub use self::packet::{AplHeader, AplDataPacket, AplAckPacket, AplErrorPacket, AplRequestPacket};

const APL_MAX_PACKET_SIZE: usize = 1024;

pub struct AplStream {
    rx: mpsc::Receiver<AplMessage>,
    tx: mpsc::Sender<AplMessage>,
    block_number: u16,
    retries: usize,
    max_retries: usize,
    max_reconnects: usize,
}

impl AplStream {
    pub fn new(buffer: usize) -> (Self, mpsc::Sender<AplMessage>) {
        let (tx1, rx1) = mpsc::channel(buffer);
        let (tx2, rx2) = mpsc::channel(buffer);
        
        (Self {
            rx: rx1,
            tx: tx2,
            block_number: 0,
            retries: 0,
            max_retries: 3,
            max_reconnects: 3,
        }, tx1)
    }

    pub fn create_request(
        &mut self,
        request_type: AplRequestType,
        block_size: usize,
        timeout: usize,
        command: u8,
        offset: usize,
        size: usize,
    ) -> Result<BytesMut, Error> {
        let mut packet = BytesMut::with_capacity(APL_MAX_PACKET_SIZE);
        
        packet.put_u8(request_type as u8);
        packet.put_u16_le(block_size as u16);
        packet.put_u16_le(timeout as u16);
        packet.put_u8(command);
        packet.put_u32_le(offset as u32);
        packet.put_u32_le(size as u32);
        
        Ok(packet)
    }

    async fn handle_data(&mut self, msg: AplMessage) -> Result<(), Error> {
        if msg.block_number != self.block_number {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid block number"));
        }

        let response = AplMessage {
            packet_type: AplRequestType::Ack,
            block_number: self.block_number,
            data: Vec::new(),
        };
        
        self.tx.send(response).await?;
        self.block_number += 1;
        self.retries = 0;
        
        Ok(())
    }

    async fn handle_ack(&mut self, msg: AplMessage) -> Result<(), Error> {
        if msg.block_number != self.block_number {
            return Err(Error::new(ErrorKind::InvalidData, "Invalid ACK block number"));
        }

        self.block_number += 1;
        self.retries = 0;
        Ok(())
    }

    async fn handle_error(&mut self, msg: AplMessage) -> Result<(), Error> {
        if self.retries >= self.max_retries {
            return Err(Error::new(ErrorKind::Other, "Max retries exceeded"));
        }
        
        self.retries += 1;
        Err(Error::new(ErrorKind::Other, format!("Protocol error: {:?}", msg.data)))
    }

    pub async fn process_message(&mut self, msg: AplMessage) -> Result<(), Error> {
        match msg.packet_type {
            AplRequestType::Data => self.handle_data(msg).await,
            AplRequestType::Ack => self.handle_ack(msg).await,
            AplRequestType::Error => self.handle_error(msg).await,
            _ => Err(Error::new(ErrorKind::InvalidData, "Unsupported message type")),
        }
    }
}

impl Stream for AplStream {
    type Item = Result<AplMessage, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx).map(|opt| opt.map(Ok))
    }
}
