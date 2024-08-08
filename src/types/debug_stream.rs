use std::collections::VecDeque;

pub use crate::traits::{Decoder, Encoder};
pub use crate::types::encoder::EncoderBufferringStatus;
pub use crate::types::encoder::EncoderError;
pub use crate::types::encoder::EncoderOk;

use tokio::io::{AsyncRead, AsyncWrite};

// fake stream for testing
// it has an internal queue for holding messages (we are sending messages to ourselves)
// it has an internal buffer for holding the current message being written
//
// statuses/states are not done yet
pub struct LDCodecDebugStream {
    queue: tokio::sync::Mutex<VecDeque<Vec<u8>>>,
    current_write_buffer: tokio::sync::Mutex<Vec<u8>>,
}

impl Default for LDCodecDebugStream {
    fn default() -> Self {
        LDCodecDebugStream {
            queue: tokio::sync::Mutex::new(VecDeque::new()),
            current_write_buffer: tokio::sync::Mutex::new(Vec::new()),
        }
    }
}

// implement decoder and encoder for fake stream
impl AsyncRead for LDCodecDebugStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        //try lock
        let mut queue = match self.queue.try_lock() {
            Ok(queue) => queue,
            Err(_) => return std::task::Poll::Pending,
        };
        println!("poll_read, awaiting queue is: {:?}", queue.len());

        // we are reading from the queue, we have to imagine there is either a message or not
        // partial reads are not done yet
        if let Some(frame) = queue.pop_front() {
            // weird reading mechanism but we do not care, read is
            // going to fire up untill we read nothing
            // it is how the tokio::io::ReadBuf works more or less
            let free_space = buf.remaining();
            println!("poll_read, free_space is: {:?}", free_space);
            // if we have more data than the buffer can hold, we write whatever we can
            // there is no way to hint the buffer to resize
            if free_space <= frame.len() {
                buf.put_slice(&frame[..free_space]);
                queue.push_front(frame[free_space..].to_vec());
                std::task::Poll::Ready(Ok(()))
            } else {
                // if we have less data than the buffer can hold, we write everything
                buf.put_slice(&frame);
                std::task::Poll::Ready(Ok(()))
            }
        } else {
            std::task::Poll::Pending
        }
    }
}

impl AsyncWrite for LDCodecDebugStream {
    fn is_write_vectored(&self) -> bool {
        false
    }
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        // yeah, only full writes are supported for now
        // we can lock or not, technically pending means we have
        // to set up waker but it does not matter for now
        match self.current_write_buffer.try_lock() {
            // poll_write is called multiple times, we do not know when caller is going to stop
            Ok(mut current_write_buffer) => {
                current_write_buffer.extend_from_slice(buf);
                println!("poll_write, currentbuffer is: {:?}", current_write_buffer.len());
                std::task::Poll::Ready(Ok(buf.len()))
            }
            Err(_) => std::task::Poll::Pending,
        }
    }

    // this is called when the caller is done writing
    // creates the message and pushes it to the queue
    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.current_write_buffer.try_lock() {
            Ok(mut current_write_buffer) => {
                let mut queue = match self.queue.try_lock() {
                    Ok(queue) => queue,
                    Err(_) => return std::task::Poll::Pending,
                };
                queue.push_back(current_write_buffer.clone());
                current_write_buffer.clear();

                println!("poll_flush: queue is: {:?}", queue.len());
                // lets print the debug data
                for each in queue.iter() {
                    println!(" => element len   : {:?}", each.len());
                    println!(" => element data  : {:x?}", each);
                }
                std::task::Poll::Ready(Ok(()))
            }
            Err(_) => std::task::Poll::Pending,
        }
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
}

// ugh
impl Unpin for LDCodecDebugStream {}
unsafe impl Send for LDCodecDebugStream {}
unsafe impl Sync for LDCodecDebugStream {}
