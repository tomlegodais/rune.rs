use std::collections::{HashMap, HashSet};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct RegionId(pub u16);

impl RegionId {
    pub fn from_coords(x: u16, y: u16) -> Self {
        Self((x << 8) | y)
    }

    pub fn x(self) -> u16 {
        (self.0 >> 8) & 0xFF
    }

    pub fn y(self) -> u16 {
        self.0 & 0xFF
    }

    pub fn to(self, other: RegionId) -> impl Iterator<Item = RegionId> {
        (self.x()..=other.x())
            .flat_map(move |x| (self.y()..=other.y()).map(move |y| RegionId::from_coords(x, y)))
    }
}

#[allow(dead_code)]
pub struct Region {
    pub id: RegionId,
    pub players: HashSet<usize>,
    pub npcs: HashSet<usize>,
}

impl Region {
    fn new(id: RegionId) -> Self {
        Self {
            id,
            players: HashSet::new(),
            npcs: HashSet::new(),
        }
    }
}

#[derive(Default)]
pub struct RegionMap {
    regions: HashMap<RegionId, Region>,
}

impl RegionMap {
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
        }
    }

    pub fn add_player(&mut self, player_id: usize, region_id: RegionId) {
        self.region_mut(region_id).players.insert(player_id);
    }

    pub fn remove_player(&mut self, player_id: usize, region_id: RegionId) {
        if let Some(region) = self.regions.get_mut(&region_id) {
            region.players.remove(&player_id);
        }
    }

    pub fn update_player_region(
        &mut self,
        player_index: usize,
        old_region: RegionId,
        new_region: RegionId,
    ) {
        if old_region == new_region {
            return;
        }

        if let Some(region) = self.regions.get_mut(&old_region) {
            region.players.remove(&player_index);
        }

        self.region_mut(new_region).players.insert(player_index);
    }

    pub fn add_npc(&mut self, npc_id: usize, region_id: RegionId) {
        self.region_mut(region_id).npcs.insert(npc_id);
    }

    pub fn remove_npc(&mut self, npc_id: usize, region_id: RegionId) {
        if let Some(region) = self.regions.get_mut(&region_id) {
            region.npcs.remove(&npc_id);
        }
    }

    pub fn update_npc_region(
        &mut self,
        npc_index: usize,
        old_region: RegionId,
        new_region: RegionId,
    ) {
        if old_region == new_region {
            return;
        }

        if let Some(region) = self.regions.get_mut(&old_region) {
            region.npcs.remove(&npc_index);
        }

        self.region_mut(new_region).npcs.insert(npc_index);
    }

    fn region_mut(&mut self, id: RegionId) -> &mut Region {
        self.regions.entry(id).or_insert_with(|| Region::new(id))
    }
}
