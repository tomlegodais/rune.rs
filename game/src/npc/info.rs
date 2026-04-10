use net::Outbox;

use crate::{
    npc::{NpcSnapshot, encode_npc_info},
    world::Position,
};

pub struct NpcInfo {
    outbox: Outbox,

    pub local_npcs: Vec<usize>,
    pub pending_add: Vec<NpcSnapshot>,
    pub pending_remove: Vec<usize>,
}

impl NpcInfo {
    pub fn new(outbox: Outbox) -> Self {
        Self {
            outbox,
            local_npcs: Vec::new(),
            pending_add: Vec::new(),
            pending_remove: Vec::new(),
        }
    }

    pub fn sync(&mut self, snapshots: &[NpcSnapshot], is_within_view: impl Fn(Position) -> bool) {
        self.pending_add.clear();
        self.pending_remove.clear();

        for &idx in &self.local_npcs {
            let alive = snapshots.iter().find(|s| s.index == idx);
            if !alive.is_some_and(|s| is_within_view(s.position)) {
                self.pending_remove.push(idx);
            }
        }

        for snapshot in snapshots {
            if !self.local_npcs.contains(&snapshot.index) && is_within_view(snapshot.position) {
                self.pending_add.push(snapshot.clone());
            }
        }
    }

    pub async fn flush(&mut self, snapshots: &[NpcSnapshot], player_pos: Position) {
        let frame = encode_npc_info(self, snapshots, player_pos);
        let _ = self.outbox.send(frame).await;
    }

    pub fn reset(&mut self) {
        self.local_npcs.retain(|idx| !self.pending_remove.contains(idx));
        for snapshot in self.pending_add.drain(..) {
            self.local_npcs.push(snapshot.index);
        }
    }
}
