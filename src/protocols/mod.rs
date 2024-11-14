pub mod apl;
pub mod lpl;

use tokio::io::{AsyncRead, AsyncWrite};

pub trait ProtocolStream {
    type Message;
    
    async fn process_message(&mut self, message: Self::Message) -> Result<(), crate::error::Error>;
    async fn send_message<T: AsyncRead + AsyncWrite + Unpin>(
        &mut self,
        stream: &mut T,
        message: Self::Message,
    ) -> Result<(), crate::error::Error>;
}
