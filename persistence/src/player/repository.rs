use super::entity::skills::SkillEntry;
use super::entity::{appearance, player, skills};
use async_trait::async_trait;
use sea_orm::prelude::Expr;
use sea_orm::*;
use shaku::{Component, Interface};

pub struct PlayerData {
    pub player_id: i64,
    pub x: i32,
    pub y: i32,
    pub plane: i32,
    pub male: bool,
    pub look: [u16; 7],
    pub colors: [u8; 5],
    pub levels: [u8; 24],
    pub xp: [u32; 24],
}

#[async_trait]
pub trait PlayerRepository: Interface {
    async fn find_by_account_id(&self, account_id: i64) -> Result<Option<PlayerData>, DbErr>;
    async fn create_default(&self, account_id: i64) -> Result<PlayerData, DbErr>;
    async fn save(&self, data: &PlayerData) -> Result<(), DbErr>;
}

#[derive(Component)]
#[shaku(interface = PlayerRepository)]
pub struct PgPlayerRepository {
    #[shaku(default)]
    db: DatabaseConnection,
}

impl PlayerData {
    fn from_models(
        player: player::Model,
        appearance: appearance::Model,
        skill_model: skills::Model,
    ) -> Result<Self, DbErr> {
        let skill_entries: Vec<SkillEntry> = serde_json::from_value(skill_model.skills)
            .map_err(|e| DbErr::Type(e.to_string()))?;

        let mut levels = [1u8; 24];
        let mut xp = [0u32; 24];
        for (i, entry) in skill_entries.iter().enumerate().take(24) {
            levels[i] = entry.level;
            xp[i] = entry.xp;
        }

        let look: [u16; 7] = appearance.look.iter()
            .map(|&v| v as u16)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| DbErr::Type("invalid look array length".to_string()))?;

        let colors: [u8; 5] = appearance.colors.iter()
            .map(|&v| v as u8)
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| DbErr::Type("invalid colors array length".to_string()))?;

        Ok(PlayerData {
            player_id: player.id,
            x: player.x,
            y: player.y,
            plane: player.plane,
            male: appearance.male,
            look,
            colors,
            levels,
            xp,
        })
    }
}

#[async_trait]
impl PlayerRepository for PgPlayerRepository {
    async fn find_by_account_id(&self, account_id: i64) -> Result<Option<PlayerData>, DbErr> {
        let Some(player) = player::Entity::find()
            .filter(player::Column::AccountId.eq(account_id))
            .one(&self.db)
            .await?
        else {
            return Ok(None);
        };

        let appearance = appearance::Entity::find_by_id(player.id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("player_appearance".to_string()))?;

        let skills = skills::Entity::find_by_id(player.id)
            .one(&self.db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound("player_skills".to_string()))?;

        PlayerData::from_models(player, appearance, skills).map(Some)
    }

    async fn create_default(&self, account_id: i64) -> Result<PlayerData, DbErr> {
        let player = player::ActiveModel {
            account_id: Set(account_id),
            ..Default::default()
        }
            .insert(&self.db)
            .await?;

        let appearance = appearance::ActiveModel {
            player_id: Set(player.id),
            ..Default::default()
        }
            .insert(&self.db)
            .await?;

        let skills = skills::ActiveModel {
            player_id: Set(player.id),
            ..Default::default()
        }
            .insert(&self.db)
            .await?;

        PlayerData::from_models(player, appearance, skills)
    }

    async fn save(&self, data: &PlayerData) -> Result<(), DbErr> {
        let skill_entries: Vec<SkillEntry> = data.levels.iter()
            .zip(data.xp.iter())
            .map(|(&level, &xp)| SkillEntry { level, xp })
            .collect();

        let skills_json = serde_json::to_value(&skill_entries)
            .map_err(|e| DbErr::Type(e.to_string()))?;

        player::Entity::update_many()
            .filter(player::Column::Id.eq(data.player_id))
            .col_expr(player::Column::X, Expr::value(data.x))
            .col_expr(player::Column::Y, Expr::value(data.y))
            .col_expr(player::Column::Plane, Expr::value(data.plane))
            .exec(&self.db)
            .await?;

        appearance::Entity::update_many()
            .filter(appearance::Column::PlayerId.eq(data.player_id))
            .col_expr(appearance::Column::Male, Expr::value(data.male))
            .col_expr(appearance::Column::Look, Expr::value(data.look.map(|v| v as i16).to_vec()))
            .col_expr(appearance::Column::Colors, Expr::value(data.colors.map(|v| v as i16).to_vec()))
            .exec(&self.db)
            .await?;

        skills::Entity::update_many()
            .filter(skills::Column::PlayerId.eq(data.player_id))
            .col_expr(skills::Column::Skills, Expr::value(skills_json))
            .exec(&self.db)
            .await?;

        Ok(())
    }
}