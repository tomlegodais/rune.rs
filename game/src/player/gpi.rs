use crate::player::state::MAX_PLAYERS;
use crate::player::{
    Appearance, AppearanceEncoder, MaskBlock, MaskEncoder, MaskFlags, MoveTypeMask, PlayerInfo,
};
use crate::world::Position;
use net::{Frame, Prefix};
use tokio_util::bytes::{BufMut, BytesMut};
use util::BitsMut;

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

struct TeleportUpdate<'a> {
    from: Position,
    to: Position,
    block: Option<&'a MaskBlock<'a>>,
}

impl BitEncoder for TeleportUpdate<'_> {
    fn encode(&self, bits: &mut BytesMut, bp: &mut usize, masks: &mut BytesMut) {
        let has_masks = self.block.is_some();
        let dx = self.to.x - self.from.x;
        let dy = self.to.y - self.from.y;
        let dp = self.to.plane - self.from.plane;

        bits.put_bits(bp, 1, 1);
        bits.put_bits(bp, 1, has_masks as u32);
        bits.put_bits(bp, 2, 3);

        if dx.abs() <= 14 && dy.abs() <= 14 {
            let x = if dx < 0 { dx + 32 } else { dx } as u32;
            let y = if dy < 0 { dy + 32 } else { dy } as u32;
            bits.put_bits(bp, 1, 0);
            bits.put_bits(bp, 12, y + (x << 5) + ((dp as u32 & 0x3) << 10));
        } else {
            bits.put_bits(bp, 1, 1);
            bits.put_bits(
                bp,
                30,
                (dy as u32 & 0x3fff) + ((dx as u32 & 0x3fff) << 14) + ((dp as u32 & 0x3) << 28),
            );
        }

        if let Some(block) = self.block {
            write_masks(masks, block);
        }
    }
}

struct PlayerRemove {
    cached_hash: u32,
}

impl BitEncoder for PlayerRemove {
    fn encode(&self, bits: &mut BytesMut, bp: &mut usize, _masks: &mut BytesMut) {
        bits.put_bits(bp, 1, 1);
        bits.put_bits(bp, 1, 0);
        bits.put_bits(bp, 2, 0);
        bits.put_bits(bp, 1, 0);
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

pub fn encode(info: &mut PlayerInfo, block: &MaskBlock) -> Frame {
    let mut bits = BytesMut::new();
    let mut masks = BytesMut::new();

    write_local(info, &mut bits, &mut masks, block, false);
    write_local(info, &mut bits, &mut masks, block, true);
    write_outside(info, &mut bits, &mut masks, true);
    write_outside(info, &mut bits, &mut masks, false);

    bits.extend_from_slice(&masks);

    Frame {
        opcode: 97,
        prefix: Prefix::Short,
        payload: bits.freeze(),
    }
}

fn write_local(
    info: &mut PlayerInfo,
    bits: &mut BytesMut,
    masks: &mut BytesMut,
    block: &MaskBlock,
    active: bool,
) {
    let mut bp = bits.bits_start();
    let mut skip = 0u32;

    for idx in 1..MAX_PLAYERS {
        let state = &info[idx];
        if !state.local || (state.activity & 1 != 0) != active {
            continue;
        }

        if skip > 0 {
            skip -= 1;
            info[idx].activity |= 2;
            continue;
        }

        let removing = info.pending_remove.contains(&idx);
        let is_self = idx == info.self_id;
        let needs_update = is_self && !block.flags.is_empty();

        if removing {
            let cached_hash = info[idx].region_hash;
            PlayerRemove { cached_hash }.encode(bits, &mut bp, masks);
        } else if let Some(tele) = info[idx].teleport {
            TeleportUpdate {
                from: tele.from,
                to: tele.to,
                block: if needs_update { Some(block) } else { None },
            }
            .encode(bits, &mut bp, masks);
        } else if needs_update {
            LocalUpdate { block }.encode(bits, &mut bp, masks);
        } else {
            skip = count_skips(info, idx + 1, true, active, |i| {
                info.pending_remove.contains(&i)
                    || info[i].teleport.is_some()
                    || (i == info.self_id && !block.flags.is_empty())
            });

            Skip(skip).encode(bits, &mut bp, masks);
            info[idx].activity |= 2;
        }
    }

    bits.bits_end(bp);
}

fn write_outside(info: &mut PlayerInfo, bits: &mut BytesMut, masks: &mut BytesMut, active: bool) {
    let mut bp = bits.bits_start();
    let mut skip = 0u32;

    for idx in 1..MAX_PLAYERS {
        let state = &info[idx];
        if state.local || (state.activity & 1 != 0) != active {
            continue;
        }

        if skip > 0 {
            skip -= 1;
            info[idx].activity |= 2;
            continue;
        }

        let pending_data = info
            .pending_add
            .iter()
            .find(|p| p.id == idx)
            .map(|p| (p.position, p.appearance.clone(), p.mask_flags));

        if let Some((position, appearance, mask_flags)) = pending_data {
            let cached_hash = info[idx].region_hash;

            PlayerAdd {
                position,
                appearance: &appearance,
                mask_flags,
                cached_hash,
            }
            .encode(bits, &mut bp, masks);

            info[idx].region_hash = position.region_hash();
            info[idx].activity |= 2;
        } else {
            skip = count_skips(info, idx + 1, false, active, |i| {
                info.pending_add.iter().any(|p| p.id == i)
            });

            Skip(skip).encode(bits, &mut bp, masks);
            info[idx].activity |= 2;
        }
    }

    bits.bits_end(bp);
}

fn count_skips(
    info: &PlayerInfo,
    from: usize,
    want_local: bool,
    active: bool,
    should_break: impl Fn(usize) -> bool,
) -> u32 {
    let mut count = 0;
    for idx in from..MAX_PLAYERS {
        let state = &info[idx];
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
