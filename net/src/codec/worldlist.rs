use crate::error::SessionError;
use tokio_util::bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub struct WorldListInbound {
    pub full_update: bool,
}

#[derive(Debug)]
pub struct WorldListOutbound {
    pub bytes: Vec<u8>,
}

#[derive(Debug, Default)]
pub struct WorldListCodec;

impl Decoder for WorldListCodec {
    type Item = WorldListInbound;
    type Error = SessionError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let full_update = src.get_u8() == 0;
        Ok(Some(WorldListInbound { full_update }))
    }
}

impl Encoder<WorldListOutbound> for WorldListCodec {
    type Error = SessionError;

    fn encode(&mut self, item: WorldListOutbound, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(&item.bytes);
        Ok(())
    }
}
