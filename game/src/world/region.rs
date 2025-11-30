use std::collections::{HashMap, HashSet};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct RegionId(pub u16);

#[allow(dead_code)]
pub struct Region {
    pub id: RegionId,
    pub players: HashSet<u16>,
}

pub struct RegionMap {
    regions: HashMap<RegionId, Region>,
}

#[allow(dead_code)]
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
}

impl Region {
    fn new(id: RegionId) -> Self {
        Self {
            id,
            players: HashSet::new(),
        }
    }
}

impl RegionMap {
    pub fn new() -> Self {
        Self {
            regions: HashMap::new(),
        }
    }

    fn region_mut(&mut self, id: RegionId) -> &mut Region {
        self.regions.entry(id).or_insert_with(|| Region::new(id))
    }

    pub fn add_player(&mut self, player_id: u16, region_id: RegionId) {
        self.region_mut(region_id).players.insert(player_id);
    }

    pub fn update_player_region(
        &mut self,
        player_id: u16,
        old_region: RegionId,
        new_region: RegionId,
    ) {
        if old_region == new_region {
            return;
        }

        if let Some(region) = self.regions.get_mut(&old_region) {
            region.players.remove(&player_id);
        }

        self.region_mut(new_region).players.insert(player_id);
    }
}
