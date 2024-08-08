use crate::types::debug_stream::{EncoderError, EncoderOk};
pub use crate::types::decoder::{DecoderError, DecoderOk};

// async trait
// defines decoder. The role of a decoder is as follows:
// 1. Read data from a stream
// 2. Decode the data
//   2.1. If the data is not enough, return NotEnough and store what is read so far
//   2.2. If the data is enough, return Message
// 3. If the stream is closed, return StreamClosed
#[async_trait::async_trait]
pub trait Decoder {
    #[must_use]
    async fn try_read_decoded(&mut self) -> Result<DecoderOk, DecoderError>;
}

// async trait
// defines encoder. The role of an encoder is as follows:
// 1. Encode the data
// 2. Write the encoded data to a stream
//   2.1. If the write is partial, return PartialOk, partial
//        data means that client writes multiple times up to X size
//   2.2. If the write is full, return FullOk
// 3. If the stream is closed, return StreamClosed
#[async_trait::async_trait]
pub trait Encoder {
    #[must_use]
    async fn write_encoded(
        &mut self,
        data: Vec<u8>,
    ) -> Result<EncoderOk, EncoderError>;
    #[must_use]
    async fn write_partial(
        &mut self,
        data: Vec<u8>,
    ) -> Result<EncoderOk, EncoderError>;
}
