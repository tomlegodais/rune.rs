use crate::codec::{Js5Codec, XorCodec};
use crate::error::SessionError;
use crate::message::{Js5Inbound, Js5Outbound};
use crate::service::CacheService;
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
        let codec = Js5Codec::default();
        let xor_codec = XorCodec::new(codec);
        let mut framed = Framed::new(stream, xor_codec);

        while let Some(frame) = framed.next().await {
            let msg = frame?;

            match msg {
                Js5Inbound::FileRequest(request) => {
                    if let Ok(data) = service.get_file(&request) {
                        let outbound = Js5Outbound {
                            index: request.index,
                            archive: request.archive,
                            data,
                            urgent: request.urgent,
                        };
                        framed.send(outbound).await?;
                    }
                }

                Js5Inbound::EncryptionKey(key) => {
                    framed.codec_mut().set_xor_key(key);
                }
            }
        }

        Ok(())
    }
}
