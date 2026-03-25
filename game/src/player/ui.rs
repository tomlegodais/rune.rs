pub mod tabs {
    use crate::player::SubInterface;

    interface_group! {
        pub const ATTACK: SubInterface = SubInterface::new(884, 152, 33);
        pub const SKILLS: SubInterface = SubInterface::new(320, 153, 34);
        pub const QUESTS: SubInterface = SubInterface::new(190, 154, 35);
        pub const ACHIEVEMENTS: SubInterface = SubInterface::new(259, 155, 36);
        pub const INVENTORY: SubInterface = SubInterface::new(149, 156, 37);
        pub const EQUIPMENT: SubInterface = SubInterface::new(387, 157, 38);
        pub const PRAYER: SubInterface = SubInterface::new(271, 158, 39);
        pub const MAGIC: SubInterface = SubInterface::new(192, 159, 40);
        pub const OBJECTIVE: SubInterface = SubInterface::new(891, 160, 41);
        pub const FRIENDS: SubInterface = SubInterface::new(550, 161, 42);
        pub const IGNORES: SubInterface = SubInterface::new(551, 162, 43);
        pub const CLAN: SubInterface = SubInterface::new(589, 163, 44);
        pub const SETTINGS: SubInterface = SubInterface::new(261, 164, 45);
        pub const EMOTES: SubInterface = SubInterface::new(464, 165, 46);
        pub const MUSIC: SubInterface = SubInterface::new(187, 166, 47);
        pub const NOTES: SubInterface = SubInterface::new(34, 167, 48);
        pub const LOGOUT: SubInterface = SubInterface::new(182, 170, 51);
    }
}

pub mod orbs {
    use crate::player::SubInterface;

    interface_group! {
        pub const SUMMONING: SubInterface = SubInterface::new(747, 139, 172);
        pub const HITPOINTS: SubInterface = SubInterface::new(748, 134, 169);
        pub const PRAYER: SubInterface = SubInterface::new(749, 136, 170);
        pub const RUN_ENERGY: SubInterface = SubInterface::new(750, 137, 171);
    }
}

pub mod chat {
    use crate::player::SubInterface;

    interface_group! {
        pub const CHAT_FRAME: SubInterface = SubInterface::new(752, 142, 18);
        pub const CHAT_AREA: SubInterface = SubInterface::chatbox(137, 9);
        pub const CHAT_OPTIONS: SubInterface = SubInterface::new(751, 20, 15);
        pub const PRIVATE_CHAT_AREA: SubInterface = SubInterface::new(754, 14, 19);
    }
}
