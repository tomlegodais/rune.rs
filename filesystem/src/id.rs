use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndexId(pub(crate) u8);

impl IndexId {
    pub const MAX_INDEX: u8 = 27;

    pub const ANIMATIONS: Self = Self(0);
    pub const SKELETONS: Self = Self(1);
    pub const CONFIGS: Self = Self(2);
    pub const INTERFACES: Self = Self(3);
    pub const SOUND_EFFECTS: Self = Self(4);
    pub const MAPS: Self = Self(5);
    pub const MUSIC_TRACKS: Self = Self(6);
    pub const MODELS: Self = Self(7);
    pub const SPRITES: Self = Self(8);
    pub const TEXTURES: Self = Self(9);
    pub const BINARY: Self = Self(10);
    pub const MUSIC_JINGLES: Self = Self(11);
    pub const CLIENT_SCRIPTS: Self = Self(12);
    pub const FONT_METRICS: Self = Self(13);
    pub const VORBIS: Self = Self(14);
    pub const OGG_INSTRUMENTS: Self = Self(15);
    pub const WORLD_MAP_OLD: Self = Self(16);
    pub const DEFAULTS: Self = Self(17);
    pub const WORLD_MAP_GEOGRAPHY: Self = Self(18);
    pub const ITEMS: Self = Self(19);
    pub const NPCS: Self = Self(20);
    pub const OBJECTS: Self = Self(21);
    pub const FLOORS: Self = Self(22);
    pub const IDENTKIT: Self = Self(23);
    pub const OVERLAYS: Self = Self(24);
    pub const INVENTORIES: Self = Self(25);
    pub const WORLD_MAP: Self = Self(26);
    pub const PARTICLES: Self = Self(27);

    #[inline(always)]
    pub const fn new(id: u8) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    #[inline(always)]
    pub const fn is_reference(self) -> bool {
        self.0 == 255
    }

    #[inline]
    pub const fn is_valid_data_index(self) -> bool {
        self.0 <= Self::MAX_INDEX
    }

    pub const fn name(self) -> &'static str {
        match self.0 {
            0 => "animations",
            1 => "skeletons",
            2 => "configs",
            3 => "interfaces",
            4 => "sound_effects",
            5 => "maps",
            6 => "music_tracks",
            7 => "models",
            8 => "sprites",
            9 => "textures",
            10 => "binary",
            11 => "music_jingles",
            12 => "client_scripts",
            13 => "font_metrics",
            14 => "vorbis",
            15 => "ogg_instruments",
            16 => "world_map_old",
            17 => "defaults",
            18 => "world_map_geography",
            19 => "items",
            20 => "npcs",
            21 => "objects",
            22 => "floors",
            23 => "identkit",
            24 => "overlays",
            25 => "inventories",
            26 => "world_map",
            255 => "reference_tables",
            _ => "unknown",
        }
    }
}

impl fmt::Debug for IndexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "IndexId({})", self.0)
    }
}

impl fmt::Display for IndexId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub const REFERENCE_INDEX: IndexId = IndexId(255);

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArchiveId(pub(crate) u32);

impl ArchiveId {
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Debug for ArchiveId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArchiveId({})", self.0)
    }
}

impl fmt::Display for ArchiveId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(pub(crate) u32);

impl FileId {
    #[inline(always)]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Debug for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FileId({})", self.0)
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
