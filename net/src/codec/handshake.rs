use crate::error::SessionError;
use crate::message::{HandshakeInbound, HandshakeOutbound, HandshakeType};
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, Default)]
pub struct HandshakeCodec;

impl Decoder for HandshakeCodec {
    type Item = HandshakeInbound;
    type Error = SessionError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        let opcode = src.get_u8();
        let handshake_type = HandshakeType::try_from(opcode)
            .map_err(|_| SessionError::InvalidHandshakeOpcode(opcode))?;

        match handshake_type {
            HandshakeType::Js5 => {
                if src.len() < 4 {
                    return Ok(None);
                }
                let client_version = src.get_u32();
                Ok(Some(HandshakeInbound::Js5 { client_version }))
            }

            HandshakeType::WorldList => {
                if src.is_empty() {
                    return Ok(None);
                }
                let full_update = src.get_u8() == 0;
                Ok(Some(HandshakeInbound::WorldList { full_update }))
            }

            HandshakeType::Login => Ok(Some(HandshakeInbound::Login)),
        }
    }
}

impl Encoder<HandshakeOutbound> for HandshakeCodec {
    type Error = SessionError;

    fn encode(&mut self, item: HandshakeOutbound, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            HandshakeOutbound::Response(response) => {
                dst.put_u8(response as u8);
            }
        }

        Ok(())
    }
}
