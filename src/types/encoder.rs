// status of the encoder, commonly returned from read/write functions
#[derive(Debug, Clone, PartialEq, Copy)]
pub struct EncoderBufferringStatus {
    // requested size of the write
    pub(crate) requested_size: u64,
    // currently buffered size
    pub(crate) internal_size: u64,
    // maximum message size
    pub(crate) max_message_size: u64,
}

pub enum EncoderOk {
    // we wrote some data
    // returns written bytes
    PartialOk(u64),
    // we wrote everything
    // returns written bytes
    FullOk(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EncoderError {
    // we tried to write, but stream is not writable, probably in partial read mode
    NotInWritableState,

    // we tried to write, but stream is in partial writable state, awaiting more data and flush
    InWritablePartialState,

    // we aborted partial write, returning the number of bytes in the buffer
    AbortedPartialWrite(u64),

    // we tried to write, but stream is closed, forward the error from underlying stream
    StreamClosed(String),

    // we tried to write partial data (or add more data to the buffer), but result would be too big
    PartialTooBig(EncoderBufferringStatus),

    // we tried to write, but the data is too big
    DataTooBig(EncoderBufferringStatus),

    // we tried to write, but the write failed
    WriteFailed(String),
}
