use std::{any::TypeId, future::Future, pin::Pin, sync::Arc};

use macros::{player_action, player_system};

use crate::{
    entity::Hit,
    player::{
        Clientbound, PlayerSnapshot,
        action::fire_action,
        stat::{Stat, StatManager},
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
    },
    world::{Position, World},
};

const DEATH_SEQ: u16 = 836;
const DEATH_TICKS: u16 = 5;

fn respawn() -> Position {
    Position::new(3221, 3219, 0)
}

#[player_action]
async fn death_action() {
    lock!();

    player.entity.stop();
    player.entity.face_target = None;
    player.player_info.add_mask(crate::player::FaceEntityMask(65535));
    player.combat_mut().set_combat_target(None);

    seq!(DEATH_SEQ);
    delay!(DEATH_TICKS);

    player.hitpoints_mut().revive();
    player.interaction_mut().clear();
    player.movement_mut().teleport(respawn()).await;

    seq!(0xFFFF);
    player.send_message("Oh dear, you are dead!").await;
    unlock!();
}

pub struct HitpointsManager {
    player: PlayerHandle,
    current: u8,
    dirty: bool,
    dying: bool,
}

impl HitpointsManager {
    pub fn current(&self) -> u8 {
        self.current
    }

    pub fn max(&self) -> u8 {
        self.player.stat().level(Stat::Hitpoints)
    }

    pub fn is_dying(&self) -> bool {
        self.dying
    }

    pub fn damage(&mut self, mut hit: Hit) -> bool {
        if self.dying {
            return false;
        }
        hit.damage = hit.damage.min(self.current as u16);
        self.current -= hit.damage as u8;
        self.dirty = true;
        hit.hp_ratio = (self.current as u32 * 255 / self.max() as u32) as u8;
        self.player.get_mut().hit(hit);
        self.current == 0
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn heal(&mut self, amount: u8) {
        let max = self.max();
        self.current = self.current.saturating_add(amount).min(max);
        self.dirty = true;
    }

    pub fn revive(&mut self) {
        self.current = self.max();
        self.dying = false;
        self.dirty = true;
    }

    fn on_death(&mut self) {
        self.dying = true;
        fire_action(self.player.get_mut(), death_action());
    }

    async fn flush(&mut self) {
        let xp = self.player.stat().xp(Stat::Hitpoints);
        self.player.update_stat(Stat::Hitpoints as u8, self.current, xp).await;
    }

    async fn tick_inner(&mut self) {
        if self.current == 0 && !self.dying {
            self.on_death();
        } else if self.dirty {
            self.dirty = false;
            self.flush().await;
        }
    }
}

#[player_system]
impl PlayerSystem for HitpointsManager {
    type TickContext = ();

    fn dependencies() -> Vec<TypeId> {
        vec![TypeId::of::<StatManager>()]
    }

    fn create(ctx: &PlayerInitContext) -> Self {
        let current = if ctx.player_data.current_hp > 0 {
            ctx.player_data.current_hp
        } else {
            ctx.player_data.levels[Stat::Hitpoints as usize]
        };

        Self {
            player: ctx.player,
            current,
            dirty: false,
            dying: false,
        }
    }

    fn on_login<'a>(
        &'a mut self,
        _player: &'a mut crate::player::Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(self.flush())
    }

    fn tick_context(_: &Arc<World>, _: &PlayerSnapshot) {}

    fn tick<'a>(&'a mut self, _ctx: &'a ()) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(self.tick_inner())
    }

    fn persist(&self, data: &mut persistence::PlayerData) {
        data.current_hp = self.current;
    }
}
