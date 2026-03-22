use std::collections::{HashMap, HashSet};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct RegionId(pub u16);

#[allow(dead_code)]
pub struct Region {
    pub id: RegionId,
    pub players: HashSet<usize>,
}

#[derive(Default)]
pub struct RegionMap {
    regions: HashMap<RegionId, Region>,
}

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
        player_id: usize,
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
