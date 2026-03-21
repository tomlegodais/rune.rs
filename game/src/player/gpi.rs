use crate::player::{
    Appearance, AppearanceEncoder, MaskBlock, MaskEncoder, MaskFlags, MoveTypeMask, Viewport,
};
use crate::world::Position;
use net::{Frame, Prefix};
use tokio_util::bytes::{BufMut, BytesMut};
use util::BitsMut;

const MAX_PLAYERS: usize = 2048;

trait BitEncoder {
    fn encode(&self, bits: &mut BytesMut, bp: &mut usize, masks: &mut BytesMut);
}

struct Skip(u32);

impl BitEncoder for Skip {
    fn encode(&self, bits: &mut BytesMut, bp: &mut usize, _masks: &mut BytesMut) {
        bits.put_bits(bp, 1, 0);
        match self.0 {
            0 => bits.put_bits(bp, 2, 0),
            1..=31 => {
                bits.put_bits(bp, 2, 1);
                bits.put_bits(bp, 5, self.0);
            }
            32..=255 => {
                bits.put_bits(bp, 2, 2);
                bits.put_bits(bp, 8, self.0);
            }
            _ => {
                bits.put_bits(bp, 2, 3);
                bits.put_bits(bp, 11, self.0);
            }
        }
    }
}

struct LocalUpdate<'a> {
    block: &'a MaskBlock<'a>,
}

impl BitEncoder for LocalUpdate<'_> {
    fn encode(&self, bits: &mut BytesMut, bp: &mut usize, masks: &mut BytesMut) {
        bits.put_bits(bp, 1, 1);
        bits.put_bits(bp, 1, 1);
        bits.put_bits(bp, 2, 0);
        write_masks(masks, self.block);
    }
}

struct RegionUpdate {
    current_hash: u32,
    cached_hash: u32,
}

impl BitEncoder for RegionUpdate {
    fn encode(&self, bits: &mut BytesMut, bp: &mut usize, _masks: &mut BytesMut) {
        let delta_plane = ((self.current_hash >> 16) - (self.cached_hash >> 16)) & 0x3;
        let delta_x = ((self.current_hash >> 8 & 0xFF) - (self.cached_hash >> 8 & 0xFF)) & 0xFF;
        let delta_y = ((self.current_hash & 0xFF) - (self.cached_hash & 0xFF)) & 0xFF;
        let delta = (delta_plane << 16) | (delta_x << 8) | delta_y;
        bits.put_bits(bp, 2, 3);
        bits.put_bits(bp, 18, delta);
    }
}

struct PlayerAdd<'a> {
    position: Position,
    appearance: &'a Appearance,
    mask_flags: MaskFlags,
    cached_hash: u32,
}

impl BitEncoder for PlayerAdd<'_> {
    fn encode(&self, bits: &mut BytesMut, bp: &mut usize, masks: &mut BytesMut) {
        let current_hash = self.position.region_hash();
        let needs_hash_update = current_hash != self.cached_hash;

        bits.put_bits(bp, 1, 1);
        bits.put_bits(bp, 2, 0);
        bits.put_bits(bp, 1, needs_hash_update as u32);

        if needs_hash_update {
            RegionUpdate {
                current_hash,
                cached_hash: self.cached_hash,
            }
            .encode(bits, bp, masks);
        }

        bits.put_bits(bp, 6, self.position.x as u32 & 0x3F);
        bits.put_bits(bp, 6, self.position.y as u32 & 0x3F);
        bits.put_bits(bp, 1, 1);

        let flags = self.mask_flags | MaskFlags::APPEARANCE | MaskFlags::MOVE_TYPE;
        let appearance_encoder = AppearanceEncoder::new(self.appearance);
        let add_block = MaskBlock {
            flags: &flags,
            move_type: &MoveTypeMask,
            appearance: &appearance_encoder,
        };

        write_masks(masks, &add_block);
    }
}

pub fn encode(viewport: &mut Viewport, self_index: usize, block: &MaskBlock) -> Frame {
    let mut bits = BytesMut::new();
    let mut masks = BytesMut::new();

    write_local(viewport, &mut bits, &mut masks, self_index, block, false);
    write_local(viewport, &mut bits, &mut masks, self_index, block, true);
    write_outside(viewport, &mut bits, &mut masks, true);
    write_outside(viewport, &mut bits, &mut masks, false);

    bits.extend_from_slice(&masks);

    Frame {
        opcode: 97,
        prefix: Prefix::Short,
        payload: bits.freeze(),
    }
}

fn write_local(
    viewport: &mut Viewport,
    bits: &mut BytesMut,
    masks: &mut BytesMut,
    self_index: usize,
    block: &MaskBlock,
    active: bool,
) {
    let mut bp = bits.bits_start();
    let mut skip = 0u32;

    for idx in 1..MAX_PLAYERS {
        let state = &viewport.players[idx];
        if !state.local || (state.activity & 1 != 0) != active {
            continue;
        }

        if skip > 0 {
            skip -= 1;
            viewport.players[idx].activity |= 2;
            continue;
        }

        let needs_update = idx == self_index && !block.flags.is_empty();
        if needs_update {
            LocalUpdate { block }.encode(bits, &mut bp, masks);
        } else {
            skip = count_skips(viewport, idx + 1, true, active, |i| {
                i == self_index && !block.flags.is_empty()
            });

            Skip(skip).encode(bits, &mut bp, masks);
            viewport.players[idx].activity |= 2;
        }
    }

    bits.bits_end(bp);
}

fn write_outside(viewport: &mut Viewport, bits: &mut BytesMut, masks: &mut BytesMut, active: bool) {
    let mut bp = bits.bits_start();
    let mut skip = 0u32;

    for idx in 1..MAX_PLAYERS {
        let state = &viewport.players[idx];
        if state.local || (state.activity & 1 != 0) != active {
            continue;
        }

        if skip > 0 {
            skip -= 1;
            viewport.players[idx].activity |= 2;
            continue;
        }

        let pending_data = viewport
            .pending_add
            .iter()
            .find(|p| p.id == idx)
            .map(|p| (p.position, p.appearance.clone(), p.mask_flags));

        if let Some((position, appearance, mask_flags)) = pending_data {
            let cached_hash = viewport.players[idx].region_hash;

            PlayerAdd {
                position,
                appearance: &appearance,
                mask_flags,
                cached_hash,
            }
            .encode(bits, &mut bp, masks);

            viewport.players[idx].region_hash = position.region_hash();
            viewport.players[idx].activity |= 2;
        } else {
            skip = count_skips(viewport, idx + 1, false, active, |i| {
                viewport.pending_add.iter().any(|p| p.id == i)
            });

            Skip(skip).encode(bits, &mut bp, masks);
            viewport.players[idx].activity |= 2;
        }
    }

    bits.bits_end(bp);
}

fn count_skips(
    viewport: &Viewport,
    from: usize,
    want_local: bool,
    active: bool,
    should_break: impl Fn(usize) -> bool,
) -> u32 {
    let mut count = 0;
    for idx in from..MAX_PLAYERS {
        let state = &viewport.players[idx];
        if state.local != want_local || (state.activity & 1 != 0) != active {
            continue;
        }
        if should_break(idx) {
            break;
        }
        count += 1;
    }
    count
}

fn write_masks(masks: &mut BytesMut, block: &MaskBlock) {
    let wire = block.flags.wire_value();

    if wire > 128 {
        masks.put_u8((wire & 0xFF) as u8);
        masks.put_u8((wire >> 8) as u8);
    } else {
        masks.put_u8(wire as u8);
    }

    encode_if_set(masks, block.flags, block.move_type);
    encode_if_set(masks, block.flags, block.appearance);
}

fn encode_if_set<E: MaskEncoder>(out: &mut BytesMut, flags: &MaskFlags, encoder: &E) {
    if flags.contains(E::flag()) {
        encoder.encode_mask(out);
    }
}
