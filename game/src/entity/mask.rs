use tokio_util::bytes::{BufMut, BytesMut};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct MaskFlags(pub u32);

impl MaskFlags {
    pub const EMPTY: Self = Self(0);

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for MaskFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

pub trait Mask {
    fn flag(&self) -> MaskFlags;
    fn encode(&self, out: &mut BytesMut);
}

impl<T: Mask + ?Sized> Mask for &T {
    fn flag(&self) -> MaskFlags {
        (*self).flag()
    }

    fn encode(&self, out: &mut BytesMut) {
        (*self).encode(out)
    }
}

pub struct MaskConfig {
    pub order: &'static [MaskFlags],
    pub extended: &'static [(u32, MaskFlags)],
}

#[derive(Clone)]
struct EncodedMask {
    flag: MaskFlags,
    data: Vec<u8>,
}

#[derive(Clone)]
pub struct MaskBlock {
    config: &'static MaskConfig,
    flags: MaskFlags,
    masks: Vec<EncodedMask>,
}

impl MaskBlock {
    pub fn new(config: &'static MaskConfig) -> Self {
        Self {
            config,
            flags: MaskFlags::EMPTY,
            masks: Vec::new(),
        }
    }

    pub fn write(&self, out: &mut BytesMut) {
        let mut wire = self.flags.0;
        for &(threshold, flag) in self.config.extended {
            if wire > threshold {
                wire |= flag.0;
            }
        }

        out.put_u8(wire as u8);
        for (i, &(threshold, _)) in self.config.extended.iter().enumerate() {
            if wire > threshold {
                out.put_u8((wire >> (8 * (i + 1))) as u8);
            }
        }

        for &order_flag in self.config.order {
            if !self.flags.contains(order_flag) {
                continue;
            }
            if let Some(mask) = self.masks.iter().find(|m| m.flag == order_flag) {
                out.put_slice(&mask.data);
            }
        }
    }

    pub fn add(&mut self, mask: impl Mask) {
        let flag = mask.flag();
        let mut buf = BytesMut::new();
        mask.encode(&mut buf);

        self.flags = self.flags | flag;
        self.masks = self
            .masks
            .drain(..)
            .filter(|m| m.flag != flag)
            .chain(std::iter::once(EncodedMask {
                flag,
                data: buf.to_vec(),
            }))
            .collect();
    }

    pub fn extend(&mut self, masks: &[&dyn Mask]) {
        masks.iter().for_each(|m| self.add(*m));
    }

    pub fn clear(&mut self) {
        self.flags = MaskFlags::EMPTY;
        self.masks.clear();
    }

    pub fn has(&self, flag: MaskFlags) -> bool {
        self.flags.contains(flag)
    }

    pub fn is_empty(&self) -> bool {
        self.flags.is_empty()
    }
}
