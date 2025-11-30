use crate::crypto::StreamCipher;
use crate::{GameMessage, MessageType, SessionError};
use tokio_util::bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

const SIZES: [i16; 256] = {
    let mut a = [-3; 256];

    a[25] = 4;
    a[19] = 2;
    a[14] = 7;
    a[66] = 8;
    a[38] = 8;
    a[50] = -1;
    a[21] = -1;
    a[1] = 2;
    a[28] = 8;
    a[49] = 6;
    a[54] = 12;
    a[5] = 5;
    a[40] = 3;
    a[81] = 4;
    a[53] = -1;
    a[72] = -1;
    a[26] = 7;
    a[68] = 3;
    a[4] = -1;
    a[75] = 16;
    a[47] = 3;
    a[8] = 8;
    a[44] = -1;
    a[6] = 8;
    a[15] = -1;
    a[39] = 7;
    a[56] = -1;
    a[23] = 3;
    a[64] = 8;
    a[80] = 7;
    a[71] = 2;
    a[13] = 3;
    a[76] = 3;
    a[18] = 6;
    a[55] = 16;
    a[52] = -1;
    a[41] = 3;
    a[61] = 2;
    a[20] = 8;
    a[70] = 8;
    a[31] = 3;
    a[69] = 0;
    a[9] = -1;
    a[73] = 2;
    a[34] = 11;
    a[59] = 18;
    a[3] = -1;
    a[65] = 3;
    a[30] = 3;
    a[42] = -1;
    a[32] = -1;
    a[45] = 3;
    a[51] = 4;
    a[33] = 11;
    a[43] = 2;
    a[12] = 4;
    a[0] = -1;
    a[77] = 7;
    a[37] = 15;
    a[24] = -1;
    a[48] = 1;
    a[79] = -1;
    a[63] = -1;
    a[62] = 8;
    a[7] = 7;
    a[10] = 7;
    a[2] = -1;
    a[11] = 7;
    a[78] = -1;
    a[60] = 3;
    a[29] = 7;
    a[35] = 3;
    a[27] = -1;
    a[74] = 0;
    a[67] = 7;
    a[22] = 4;
    a[36] = 3;
    a[17] = 0;
    a[58] = 6;
    a[57] = 4;
    a[46] = 8;
    a[16] = 15;

    a
};

#[derive(Debug)]
enum State {
    Opcode,
    Size {
        opcode: u8,
        size_marker: i16,
    },
    Payload {
        opcode: u8,
        ty: MessageType,
        size: usize,
    },
}

#[derive(Debug)]
pub struct GameCodec<CIn, COut> {
    in_cipher: CIn,
    out_cipher: COut,
    state: State,
}

impl<CIn, COut> GameCodec<CIn, COut>
where
    CIn: StreamCipher,
    COut: StreamCipher,
{
    pub fn new(in_cipher: CIn, out_cipher: COut) -> Self {
        Self {
            in_cipher,
            out_cipher,
            state: State::Opcode,
        }
    }
}

impl<CIn, COut> Decoder for GameCodec<CIn, COut>
where
    CIn: StreamCipher,
    COut: StreamCipher,
{
    type Item = GameMessage;
    type Error = SessionError;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        loop {
            match self.state {
                State::Opcode => {
                    if src.len() < 1 {
                        return Ok(None);
                    }

                    let encrypted = src.get_u8();
                    let opcode = encrypted.wrapping_sub(self.in_cipher.next_u8());
                    let size_marker = SIZES[opcode as usize];

                    if size_marker == -3 {
                        return Err(SessionError::InvalidMessageSize(opcode));
                    }

                    if size_marker < 0 {
                        self.state = State::Size {
                            opcode,
                            size_marker,
                        };
                    } else {
                        self.state = State::Payload {
                            opcode,
                            ty: MessageType::Fixed,
                            size: size_marker as usize,
                        };
                    }
                }

                State::Size {
                    opcode,
                    size_marker,
                } => match size_marker {
                    -1 => {
                        if src.len() < 1 {
                            return Ok(None);
                        }

                        let size = src.get_u8() as usize;
                        self.state = State::Payload {
                            opcode,
                            ty: MessageType::Byte,
                            size,
                        };
                    }
                    -2 => {
                        if src.len() < 2 {
                            return Ok(None);
                        }
                        let size = src.get_u16() as usize;
                        self.state = State::Payload {
                            opcode,
                            ty: MessageType::Short,
                            size,
                        };
                    }
                    _ => unreachable!("size_marker must be -1 or -2 in Size state"),
                },

                State::Payload { opcode, ty, size } => {
                    if src.len() < size {
                        return Ok(None);
                    }

                    let payload = src.split_to(size).freeze();
                    self.state = State::Opcode;

                    return Ok(Some(GameMessage {
                        opcode,
                        ty,
                        payload,
                    }));
                }
            }
        }
    }
}

impl<CIn, COut> Encoder<GameMessage> for GameCodec<CIn, COut>
where
    CIn: StreamCipher,
    COut: StreamCipher,
{
    type Error = SessionError;

    fn encode(&mut self, msg: GameMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let GameMessage {
            opcode,
            ty,
            payload,
        } = msg;

        let encrypted = opcode.wrapping_add(self.out_cipher.next_u8());
        dst.put_u8(encrypted);

        match ty {
            MessageType::Fixed => {}
            MessageType::Byte => dst.put_u8(payload.len() as u8),
            MessageType::Short => dst.put_u16(payload.len() as u16),
        }

        dst.extend_from_slice(&payload);
        Ok(())
    }
}
