pub mod traits;
pub mod types;

#[cfg(test)]
mod tests {
    use crate::traits::Decoder;
    use crate::traits::Encoder;
    use crate::types::debug_stream;
    use crate::types::debug_stream::LDCodecDebugStream;
    use crate::types::decoder::DecoderOk;
    use crate::types::ldcodec::LDCodec;

    #[tokio::test]
    async fn it_works() {
        let mut codec: LDCodec<LDCodecDebugStream> = LDCodec::new(debug_stream::LDCodecDebugStream::default(), 64);

        let data = vec![0xA1, 0xB2, 0xC3, 0xD4, 0xA5];
        match codec.write_encoded(data).await {
            Ok(_) => {
                println!("test: success");
            }
            Err(e) => {
                println!("test: failed: {:?}", e);
            }
        }
        match codec.try_read_decoded().await {
            Ok(msg) => match msg {
                DecoderOk::Message(data) => {
                    println!("test: success, proper message available!");
                    println!(" > size : {}", data.len());
                    println!(" > data : {:x?}", data);
                }
                DecoderOk::StreamEmpty => {
                    println!("test: success, stream empty");
                }
                DecoderOk::NotEnough(size) => {
                    println!("test: success, not enough data: {:?}", size);
                }
            },
            Err(e) => {
                println!("test: failed: {:?}", e);
            }
        }
    }
}
