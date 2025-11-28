use crate::codec::{Js5Codec, Js5Inbound, Js5Outbound};
use crate::error::SessionError;
use crate::service::cache::CacheService;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct Js5Handler;

impl Js5Handler {
    pub async fn run(
        stream: TcpStream,
        service: Arc<CacheService>,
    ) -> anyhow::Result<(), SessionError> {
        let codec = Js5Codec::new();
        let mut framed = Framed::new(stream, codec);

        while let Some(frame) = framed.next().await {
            let msg = frame?;

            match msg {
                Js5Inbound::FileRequest(request) => {
                    if let Ok(bytes) = service.serve(&request) {
                        let outbound = Js5Outbound { bytes };
                        framed.send(outbound).await?;
                    }
                }

                Js5Inbound::EncryptionKey(key) => {
                    let codec = framed.codec_mut();
                    codec.xor_key = key;
                }
            }
        }

        Ok(())
    }
}
