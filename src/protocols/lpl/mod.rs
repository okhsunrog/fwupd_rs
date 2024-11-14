use tokio::sync::mpsc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_stream::{Stream, StreamExt};
use bytes::{Buf, BufMut, BytesMut};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::io::{Error, ErrorKind};
use crc::{Crc, CRC_16_CCITT_FALSE};

mod types;
pub use self::types::{LplMessage, LplStream};

use crate::protocols::apl::{AplMessage, AplRequestType};

const SYN: u8 = 0x55;
const LPL_MAX_BUFFER_SIZE: usize = 1024;

pub struct LplStream {
    rx: mpsc::Receiver<LplMessage>,
    tx: mpsc::Sender<LplMessage>,
    apl_tx: mpsc::Sender<AplMessage>,
    tx_buffer: BytesMut,
    rx_buffer: BytesMut,
}

impl LplStream {
    pub fn new(
        buffer: usize,
        apl_tx: mpsc::Sender<AplMessage>,
    ) -> (Self, mpsc::Sender<LplMessage>) {
        let (tx1, rx1) = mpsc::channel(buffer);
        let (tx2, rx2) = mpsc::channel(buffer);
        
        (Self {
            rx: rx1,
            tx: tx2,
            apl_tx,
            tx_buffer: BytesMut::with_capacity(LPL_MAX_BUFFER_SIZE),
            rx_buffer: BytesMut::with_capacity(LPL_MAX_BUFFER_SIZE),
        }, tx1)
    }

    pub async fn send_request<T: AsyncWrite + Unpin>(
        &mut self,
        stream: &mut T,
        request_type: AplRequestType,
        block_size: usize,
        timeout: usize,
        command: usize,
        offset: usize,
        size: usize,
    ) -> Result<(), Error> {
        self.tx_buffer.clear();
        self.tx_buffer.put_u8(SYN);

        let mut packet = BytesMut::with_capacity(LPL_MAX_BUFFER_SIZE);
        
        // Create APL request
        let apl_request = AplMessage {
            packet_type: request_type,
            block_number: 0,
            data: vec![],
        };
        
        packet.extend_from_slice(&apl_request.to_bytes()?);

        // Calculate CRC
        let crc = Crc::<u16>::new(&CRC_16_CCITT_FALSE);
        let mut digest = crc.digest();
        digest.update(&packet);
        let checksum = digest.finalize();
        packet.put_u16_le(checksum);

        // COBS encode
        let mut encoded = vec![0; cobs::max_encoding_length(packet.len())];
        let encoded_len = cobs::encode(&packet, &mut encoded);
        
        self.tx_buffer.extend_from_slice(&encoded[..encoded_len]);
        self.tx_buffer.put_u8(0);

        stream.write_all(&self.tx_buffer).await
    }

    async fn decode_message(&self, msg: LplMessage) -> Result<AplMessage, Error> {
        let mut decoded = vec![0; msg.payload.len()];
        let decoded_len = cobs::decode(&msg.payload, &mut decoded)?;
        
        if decoded_len < 2 {
            return Err(Error::new(ErrorKind::InvalidData, "Packet too small"));
        }

        let (data, crc_bytes) = decoded.split_at(decoded_len - 2);
        let received_crc = u16::from_le_bytes([crc_bytes[0], crc_bytes[1]]);

        let crc = Crc::<u16>::new(&CRC_16_CCITT_FALSE);
        let mut digest = crc.digest();
        digest.update(data);
        let calculated_crc = digest.finalize();

        if calculated_crc != received_crc {
            return Err(Error::new(ErrorKind::InvalidData, "CRC mismatch"));
        }

        AplMessage::from_bytes(data)
    }

    pub async fn run(&mut self) {
        while let Some(msg) = self.rx.recv().await {
            match self.decode_message(msg).await {
                Ok(apl_msg) => {
                    if let Err(e) = self.apl_tx.send(apl_msg).await {
                        log::error!("Failed to forward message to APL: {}", e);
                        break;
                    }
                }
                Err(e) => {
                    log::error!("Failed to decode message: {}", e);
                }
            }
        }
    }
}

impl Stream for LplStream {
    type Item = Result<LplMessage, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx).map(|opt| opt.map(Ok))
    }
}
