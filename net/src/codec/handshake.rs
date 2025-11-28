use crate::error::SessionError;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug, TryFromPrimitive)]
#[repr(u8)]
enum HandshakeType {
    Js5 = 15,
    WorldList = 23,
    Login = 14,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u8)]
pub enum HandshakeResponse {
    Success = 0,
    OutOfDate = 6,
}

#[derive(Debug)]
pub enum HandshakeInbound {
    Js5 { client_version: u32 },
    WorldList,
    Login,
}

pub enum HandshakeOutbound {
    Response(HandshakeResponse),
}

#[derive(Debug, Default)]
pub struct HandshakeCodec;

impl Decoder for HandshakeCodec {
    type Item = HandshakeInbound;
    type Error = SessionError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let opcode = src.get_u8();
        let handshake_type = HandshakeType::try_from(opcode)
            .map_err(|_| SessionError::InvalidHandshakeOpcode(opcode))?;

        match handshake_type {
            HandshakeType::Js5 => {
                let client_version = src.get_u32();

                println!("client_version: {}", client_version);

                Ok(Some(HandshakeInbound::Js5 { client_version }))
            }

            HandshakeType::WorldList => Ok(Some(HandshakeInbound::WorldList)),
            HandshakeType::Login => Ok(Some(HandshakeInbound::Login)),
        }
    }
}

impl Encoder<HandshakeOutbound> for HandshakeCodec {
    type Error = SessionError;

    fn encode(&mut self, item: HandshakeOutbound, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            HandshakeOutbound::Response(response) => {
                println!("Writing response: {:?}", response);
                dst.put_u8(response as u8);
            }
        }

        Ok(())
    }
}
