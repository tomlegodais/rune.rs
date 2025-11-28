#![allow(dead_code, unused_variables)]

use crate::error::SessionError;
use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

#[derive(Debug)]
pub enum LoginInbound {
    _Placeholder,
}

#[derive(Debug)]
pub enum LoginOutbound {
    _Placeholder,
}

#[derive(Debug, Default)]
pub struct LoginCodec;

impl Decoder for LoginCodec {
    type Item = LoginInbound;
    type Error = SessionError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        unimplemented!()
    }
}

impl Encoder<LoginOutbound> for LoginCodec {
    type Error = SessionError;

    fn encode(&mut self, item: LoginOutbound, dst: &mut BytesMut) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
