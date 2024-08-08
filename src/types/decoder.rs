#[derive(Debug, Clone, PartialEq)]
pub enum DecoderError {
    // we tried to read, but stream is not readable, probably in partial write mode
    NotInReadableState,

    // we tried to read, but stream is in partial readable state, awaiting more data
    InReadablePartialState,

    // we aborted partial read, returning the number of bytes in the buffer
    AbortedRead(u64),

    // we tried to read, but stream is closed, forward the error from underlying stream
    StreamClosed(String),

    // we tried to read, but the read failed
    ReadFailed(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecoderOk {
    // we read the stream but its empty
    StreamEmpty,

    // we read the stream but its not enough to decode message
    NotEnough(u32),

    // we read the stream and decoded at least one message
    Message(Vec<u8>),
}
