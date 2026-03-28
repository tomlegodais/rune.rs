use crate::npc::NpcSnapshot;
use crate::world::Position;
use net::Outbox;

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
        self.local_npcs
            .retain(|&idx| match snapshots.iter().find(|s| s.index == idx) {
                Some(s) if is_within_view(s.position) => true,
                _ => {
                    self.pending_remove.push(idx);
                    false
                }
            });

        for snapshot in snapshots {
            if !self.local_npcs.contains(&snapshot.index) && is_within_view(snapshot.position) {
                self.pending_add.push(snapshot.clone());
            }
        }
    }

    pub async fn flush(&mut self, snapshots: &[NpcSnapshot], player_pos: Position) {
        let frame = crate::npc::gni::encode(self, snapshots, player_pos);
        let _ = self.outbox.send(frame).await;
    }

    pub fn reset(&mut self) {
        for snapshot in self.pending_add.drain(..) {
            self.local_npcs.push(snapshot.index);
        }
    }
}
