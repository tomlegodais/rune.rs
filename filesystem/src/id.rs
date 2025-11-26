use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndexId(pub(crate) u8);

impl IndexId {
    pub const ANIMS: Self = Self(0);
    pub const SKELETONS: Self = Self(1);
    pub const CONFIGS: Self = Self(2);
    pub const INTERFACES: Self = Self(3);
    pub const SOUNDEFFECTS: Self = Self(4);
    pub const MAPS: Self = Self(5);
    pub const MUSIC: Self = Self(6);
    pub const MODELS: Self = Self(7);
    pub const SPRITES: Self = Self(8);
    pub const TEXTURES: Self = Self(9);
    pub const HUFFMAN: Self = Self(10);
    pub const JINGLES: Self = Self(11);
    pub const CLIENTSCRIPT: Self = Self(12);
    pub const FONTMETRICS: Self = Self(13);
    pub const VORBIS: Self = Self(14);
    pub const INSTRUMENTS: Self = Self(15);
    pub const WORLDMAPDATA: Self = Self(16);
    pub const WORLDMAP: Self = Self(17);
    pub const NPCS: Self = Self(18);
    pub const ITEMS: Self = Self(19);
    pub const SEQUENCES: Self = Self(20);
    pub const SPOTANIMS: Self = Self(21);
    pub const VARBITS: Self = Self(22);
    pub const WORLDMAP_OLD: Self = Self(23);
    pub const QUICKCHAT: Self = Self(24);
    pub const QUICKCHAT_GLOBAL: Self = Self(25);
    pub const MATERIALS: Self = Self(26);
    pub const PARTICLES: Self = Self(27);
    pub const DEFAULTS: Self = Self(28);
    pub const BILLBOARDS: Self = Self(29);
    pub const NATIVES: Self = Self(30);

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

    pub const fn name(self) -> &'static str {
        match self.0 {
            0 => "anims",
            1 => "skeletons",
            2 => "configs",
            3 => "interfaces",
            4 => "soundeffects",
            5 => "maps",
            6 => "music",
            7 => "models",
            8 => "sprites",
            9 => "textures",
            10 => "huffman",
            11 => "jingles",
            12 => "clientscript",
            13 => "fontmetrics",
            14 => "vorbis",
            15 => "instruments",
            16 => "worldmapdata",
            17 => "worldmap",
            18 => "npcs",
            19 => "items",
            20 => "sequences",
            21 => "spotanims",
            22 => "varbits",
            23 => "worldmap_old",
            24 => "quickchat",
            25 => "quickchat_global",
            26 => "materials",
            27 => "particles",
            28 => "defaults",
            29 => "billboards",
            30 => "natives",
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
    pub const fn is_reference(self) -> bool {
        self.0 == 255
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
