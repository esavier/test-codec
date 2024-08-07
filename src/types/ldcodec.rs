pub use crate::traits::{Decoder, Encoder};
pub use crate::types::encoder::EncoderBufferringStatus;
pub use crate::types::encoder::EncoderError;
pub use crate::types::encoder::EncoderOk;

use std::result::Result;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::decoder::{DecoderError, DecoderOk};

#[derive(Debug, Clone, PartialEq, Copy, Eq, PartialOrd, Ord)]
pub enum CodecState {
    Ready,
    Write,
    PartialWrite,
    Read,
    PartialRead,
    Unwrittable,
    Closed,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct LDCodec<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    // stream: tokio::net::TcpStream,
    stream: S,
    wanted_bytes: u64,
    buffer: Vec<u8>,
    max_size: u64,
    state: CodecState,
}

impl<S> LDCodec<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    pub fn new(
        stream: S,
        max_size: u64,
    ) -> LDCodec<S> {
        LDCodec {
            stream,
            wanted_bytes: 0,
            buffer: Vec::new(),
            max_size,
            state: CodecState::Ready,
        }
    }
}

#[async_trait::async_trait]
impl<S> Encoder for LDCodec<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    async fn write_encoded(
        &mut self,
        data: Vec<u8>,
    ) -> Result<EncoderOk, EncoderError> {
        println!("write_encoded:: entering");
        let message_size = if self.max_size < (data.len() + self.buffer.len()).try_into().unwrap() {
            return Err(EncoderError::DataTooBig(EncoderBufferringStatus {
                requested_size: data.len() as u64,
                internal_size: 0,
                max_message_size: self.max_size,
            }));
        } else {
            data.len() as u32
        };

        match self.stream.write_u32_le(message_size).await {
            Ok(_) => {
                println!("write_encoded: wrote message size: {:?}", message_size);
            }
            Err(e) => {
                println!("write_encoded: write failed: {:?}", e);
                return Err(EncoderError::WriteFailed(e.to_string()));
            }
        };

        match self.stream.write_all(&data).await {
            Ok(_) => {
                println!("write_encoded: wrote message data: {:?}", data);
            }
            Err(e) => {
                println!("write_encoded: write failed: {:?}", e);
                return Err(EncoderError::WriteFailed(e.to_string()));
            }
        };

        self.stream.flush().await.unwrap();

        return Ok(EncoderOk::FullOk(message_size as u64));
    }

    async fn write_partial(
        &mut self,
        _data: Vec<u8>,
    ) -> Result<EncoderOk, EncoderError> {
        unimplemented!()
    }
}

#[async_trait::async_trait]
impl<S> Decoder for LDCodec<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send + Sync,
{
    async fn try_read_decoded(&mut self) -> Result<DecoderOk, DecoderError> {
        println!("try_read_decoded");
        let message_size;
        match self.stream.read_u32_le().await {
            Ok(size) => {
                message_size = size;
                println!("try_read_decoded: read message size: {:?}", message_size);
            }
            Err(e) => {
                println!("try_read_decoded: read failed: {:?}", e);
                return Err(DecoderError::ReadFailed(e.to_string()));
            }
        };

        let mut data = vec![0u8; message_size as usize];
        match self.stream.read_exact(&mut data).await {
            Ok(_) => {
                println!("try_read_decoded: read message data: {:?}", data);
            }
            Err(e) => {
                println!("try_read_decoded: read failed: {:?}", e);
                return Err(DecoderError::ReadFailed(e.to_string()));
            }
        };

        return Ok(DecoderOk::Message(data));
    }
}
