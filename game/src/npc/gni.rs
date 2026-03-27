use crate::entity::MoveStep;
use crate::npc::NpcInfo;
use crate::npc::NpcSnapshot;
use crate::world::{Direction, Position};
use net::{Frame, Prefix};
use tokio_util::bytes::BytesMut;
use util::BitsMut;

pub fn encode(info: &mut NpcInfo, snapshots: &[NpcSnapshot], player_pos: Position) -> Frame {
    let mut bits = BytesMut::new();
    let mut masks = BytesMut::new();
    let mut bp = bits.bits_start();

    bits.put_bits(&mut bp, 8, info.local_npcs.len() as u32);

    write_local(info, snapshots, &mut bits, &mut bp, &mut masks);
    write_additions(info, player_pos, &mut bits, &mut bp, &mut masks);

    if !masks.is_empty() {
        bits.put_bits(&mut bp, 15, 0x7FFF);
    }

    bits.bits_end(bp);
    bits.extend_from_slice(&masks);

    Frame {
        opcode: 77,
        prefix: Prefix::Short,
        payload: bits.freeze(),
    }
}

fn write_local(
    info: &NpcInfo,
    snapshots: &[NpcSnapshot],
    bits: &mut BytesMut,
    bp: &mut usize,
    masks: &mut BytesMut,
) {
    for &npc_id in &info.local_npcs {
        if info.pending_remove.contains(&npc_id) {
            bits.put_bits(bp, 1, 1);
            bits.put_bits(bp, 2, 3);
            continue;
        }

        let snapshot = snapshots.iter().find(|s| s.index == npc_id);
        let has_masks = snapshot.is_some_and(|s| !s.masks.is_empty());

        match snapshot.map(|s| s.move_step) {
            Some(MoveStep::Walk(walk_dir)) => {
                bits.put_bits(bp, 1, 1);
                bits.put_bits(bp, 2, 1);
                bits.put_bits(bp, 3, npc_wire_direction(walk_dir));
                bits.put_bits(bp, 1, has_masks as u32);
                if has_masks {
                    snapshot.unwrap().masks.write(masks);
                }
            }
            Some(MoveStep::Run(opcode)) => {
                let walk_dir = opcode & 0x7;
                let run_dir = (opcode >> 3) & 0x7;
                bits.put_bits(bp, 1, 1);
                bits.put_bits(bp, 2, 2);
                bits.put_bits(bp, 1, 1);
                bits.put_bits(bp, 3, walk_dir as u32);
                bits.put_bits(bp, 3, run_dir as u32);
                bits.put_bits(bp, 1, has_masks as u32);
                if has_masks {
                    snapshot.unwrap().masks.write(masks);
                }
            }
            _ if has_masks => {
                bits.put_bits(bp, 1, 1);
                bits.put_bits(bp, 2, 0);
                snapshot.unwrap().masks.write(masks);
            }
            _ => {
                bits.put_bits(bp, 1, 0);
            }
        }
    }
}

#[rustfmt::skip]
fn write_additions(
    info: &NpcInfo,
    player_pos: Position,
    bits: &mut BytesMut,
    bp: &mut usize,
    masks: &mut BytesMut,
) {
    for snapshot in &info.pending_add {
        let has_masks = !snapshot.masks.is_empty();

        let mut dx = snapshot.position.x - player_pos.x;
        let mut dy = snapshot.position.y - player_pos.y;
        if dx < 0 { dx += 32; }
        if dy < 0 { dy += 32; }

        bits.put_bits(bp, 15, snapshot.index as u32);
        bits.put_bits(bp, 1, snapshot.teleport.is_some() as u32);
        bits.put_bits(bp, 2, snapshot.position.plane as u32);
        bits.put_bits(bp, 1, has_masks as u32);
        bits.put_bits(bp, 5, dy as u32);
        bits.put_bits(bp, 5, dx as u32);
        bits.put_bits(bp, 3, npc_wire_direction(snapshot.face_direction));
        bits.put_bits(bp, 14, snapshot.npc_id as u32);

        if has_masks {
            snapshot.masks.write(masks);
        }
    }
}

fn npc_wire_direction(dir: Direction) -> u32 {
    match dir {
        Direction::North => 0,
        Direction::NorthEast => 1,
        Direction::East => 2,
        Direction::SouthEast => 3,
        Direction::South => 4,
        Direction::SouthWest => 5,
        Direction::West => 6,
        Direction::NorthWest => 7,
    }
}
