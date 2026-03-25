use tokio_util::bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

pub struct XorCodec<C> {
    inner: C,
    xor_key: u8,
}

impl<C> XorCodec<C> {
    pub fn new(codec: C) -> Self {
        Self {
            inner: codec,
            xor_key: 0,
        }
    }

    pub fn set_xor_key(&mut self, key: u8) {
        self.xor_key = key;
    }
}

impl<C, Item> Decoder for XorCodec<C>
where
    C: Decoder<Item = Item>,
{
    type Item = Item;
    type Error = C::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.inner.decode(src)
    }
}

impl<C, Item> Encoder<Item> for XorCodec<C>
where
    C: Encoder<Item>,
{
    type Error = C::Error;

    fn encode(&mut self, item: Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let start = dst.len();
        self.inner.encode(item, dst)?;

        if self.xor_key != 0 {
            for i in start..dst.len() {
                dst[i] ^= self.xor_key;
            }
        }

        Ok(())
    }
}
