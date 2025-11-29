use crate::error::SessionError;
use crate::message::{LoginInbound, LoginOutbound, LoginRequest, LoginResponse, LoginState};
use rand::Rng;
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};
use util::{decode_base37, rsa_decrypt, BufExt, EXPONENT, MODULUS};

#[derive(Debug)]
pub struct LoginCodec {
    hash: u8,
    state: LoginState,
    size: usize,
}

impl LoginCodec {
    fn new(hash: u8) -> Self {
        Self {
            hash,
            state: LoginState::Header,
            size: 0,
        }
    }

    pub fn with_random_key(hash: u8) -> (Self, i64) {
        let mut rng = rand::rng();
        let session_key: i64 = rng.random();
        (Self::new(hash), session_key)
    }
}

impl Decoder for LoginCodec {
    type Item = LoginInbound;
    type Error = SessionError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if self.state == LoginState::Header {
            if src.len() < 4 {
                return Ok(None);
            }

            self.state = LoginState::Payload;
            let r#type = src.get_u8();
            self.size = src.get_u16() as usize;

            if r#type != 16 && r#type != 18 {
                return Err(SessionError::InvalidLoginType(r#type));
            }
        }

        if self.state == LoginState::Payload {
            if src.len() < self.size {
                return Ok(None);
            }

            let version = src.get_u32();
            let _ = src.get_u8();
            let display_mode = src.get_u8();
            let _ = src.get_u16();
            let _ = src.get_u16();
            let _ = src.get_u8();
            let mut uid = [0i8; 24];
            for i in 0..uid.len() {
                uid[i] = src.get_i8();
            }

            let _ = src.get_string();
            let _ = src.get_u32();
            let toolkit_size = src.get_u8();
            src.advance(toolkit_size as usize);

            let _ = src.get_u16();
            let mut crc = [0u32; 31];
            for i in 0..crc.len() {
                crc[i] = src.get_u32();
            }

            let encrypted_size = src.get_u8() as usize;
            let encrypted_block = src.split_to(encrypted_size);
            let plain_bytes = rsa_decrypt(&encrypted_block, MODULUS, EXPONENT)?;
            let mut secure_buf = BytesMut::from(&plain_bytes[..]);
            let encrypted_type = secure_buf.get_u8();
            if encrypted_type != 10 {
                return Err(SessionError::InvalidEncryptedType(encrypted_type));
            }

            let client_key = secure_buf.get_i64();
            let server_key = secure_buf.get_i64();
            let encoded_username = secure_buf.get_i64();
            let username = decode_base37(encoded_username);
            let password = secure_buf.get_string();
            let username_hash = ((encoded_username >> 16) & 31) as u8;
            if username_hash != self.hash {
                return Err(SessionError::UsernameHashMismatch);
            }

            let item = LoginInbound::Request(LoginRequest {
                version,
                display_mode,
                crc,
                client_key,
                server_key,
                username,
                password,
            });

            return Ok(Some(item));
        }

        Ok(None)
    }
}

impl Encoder<LoginOutbound> for LoginCodec {
    type Error = SessionError;

    fn encode(&mut self, item: LoginOutbound, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_u8(item.status_code() as u8);

        match item {
            LoginOutbound::SessionKey(session_key) => dst.put_i64(session_key),
            LoginOutbound::Response(LoginResponse {
                payload: Some(payload),
                ..
            }) => {
                dst.put(payload);
            }

            LoginOutbound::Response(LoginResponse { payload: None, .. }) => {}
        }

        Ok(())
    }
}
