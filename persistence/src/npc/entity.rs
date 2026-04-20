pub mod config {
    use sea_orm::entity::prelude::*;

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "npc_configs")]
    pub struct Model {
        #[sea_orm(primary_key, auto_increment = false)]
        pub npc_id: i32,
        pub max_hp: i32,
        pub atk_level: i16,
        pub str_level: i16,
        pub def_level: i16,
        pub atk_bonus: i16,
        pub str_bonus: i16,
        pub def_stab: i16,
        pub def_slash: i16,
        pub def_crush: i16,
        pub def_magic: i16,
        pub def_ranged: i16,
        pub atk_speed: i16,
        pub atk_seq: i16,
        pub block_seq: i16,
        pub death_seq: i16,
        pub max_hit: i16,
        pub atk_range: i16,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}

pub mod spawn {
    use sea_orm::entity::prelude::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
    #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "face_direction")]
    pub enum FaceDirection {
        #[sea_orm(string_value = "north")]
        North,
        #[sea_orm(string_value = "north_east")]
        NorthEast,
        #[sea_orm(string_value = "east")]
        East,
        #[sea_orm(string_value = "south_east")]
        SouthEast,
        #[sea_orm(string_value = "south")]
        South,
        #[sea_orm(string_value = "south_west")]
        SouthWest,
        #[sea_orm(string_value = "west")]
        West,
        #[sea_orm(string_value = "north_west")]
        NorthWest,
    }

    #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
    #[sea_orm(table_name = "npc_spawns")]
    pub struct Model {
        #[sea_orm(primary_key)]
        pub id: i32,
        pub npc_id: i32,
        pub x: i32,
        pub y: i32,
        pub plane: i32,
        pub wander_radius: i16,
        pub respawn_ticks: i16,
        pub face_direction: FaceDirection,
    }

    #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
    pub enum Relation {}

    impl ActiveModelBehavior for ActiveModel {}
}
