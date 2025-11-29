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
        const OPCODE_LEN: usize = 1;
        if src.len() < OPCODE_LEN {
            return Ok(None);
        }

        let opcode = src[0];
        let handshake_type = HandshakeType::try_from(opcode)
            .map_err(|_| SessionError::InvalidHandshakeOpcode(opcode))?;

        let needed = OPCODE_LEN + handshake_type.len();
        if src.len() < needed {
            return Ok(None);
        }

        let _ = src.get_u8();
        let item = match handshake_type {
            HandshakeType::Js5 => {
                let client_version = src.get_u32();
                HandshakeInbound::Js5 { client_version }
            }

            HandshakeType::WorldList => {
                let full_update = src.get_u8() == 0;
                HandshakeInbound::WorldList { full_update }
            }

            HandshakeType::Login => {
                let hash = src.get_u8();
                HandshakeInbound::Login { hash }
            }
        };

        Ok(Some(item))
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
