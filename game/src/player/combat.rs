use std::{future::Future, pin::Pin, sync::Arc};

use macros::player_system;
use persistence::PlayerData;

use crate::{
    content::CombatTarget,
    player::{
        PlayerSnapshot,
        action::is_action_locked,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
    },
    world::World,
};

const COMBAT_STYLE_VARP: u16 = 43;
const AUTO_RETALIATE_VARP: u16 = 172;
const SPEC_ENERGY_VARP: u16 = 300;
const MAX_SPEC_ENERGY: u16 = 1000;
const SPEC_REGEN_AMOUNT: u16 = 100;
const SPEC_REGEN_TICKS: u16 = 50;

pub struct CombatManager {
    player: PlayerHandle,
    combat_style: u8,
    auto_retaliate: bool,
    spec_energy: u16,
    spec_enabled: bool,
    spec_dirty: bool,
    spec_regen_timer: u16,
    infinite_spec: bool,
    combat_target: Option<CombatTarget>,
    eat_delay: u16,
    retaliate_target: Option<CombatTarget>,
}

impl CombatManager {
    pub fn combat_style(&self) -> u8 {
        self.combat_style
    }

    pub fn set_combat_style(&mut self, style: u8) {
        self.combat_style = style;
    }

    pub fn auto_retaliate(&self) -> bool {
        self.auto_retaliate
    }

    pub fn set_auto_retaliate(&mut self, enabled: bool) {
        self.auto_retaliate = enabled;
    }

    pub fn spec_energy(&self) -> u16 {
        self.spec_energy
    }

    pub fn spec_enabled(&self) -> bool {
        self.spec_enabled
    }

    pub fn set_spec_enabled(&mut self, enabled: bool) {
        self.spec_enabled = enabled;
        self.spec_dirty = true;
    }

    pub fn drain_spec(&mut self, amount: u16) {
        if self.infinite_spec {
            return;
        }
        self.spec_energy = self.spec_energy.saturating_sub(amount);
        self.spec_dirty = true;
    }

    pub fn toggle_infinite_spec(&mut self) -> bool {
        self.infinite_spec = !self.infinite_spec;
        if self.infinite_spec {
            self.spec_energy = MAX_SPEC_ENERGY;
            self.spec_dirty = true;
        }
        self.infinite_spec
    }

    pub fn combat_target(&self) -> Option<CombatTarget> {
        self.combat_target
    }

    pub fn set_combat_target(&mut self, target: Option<CombatTarget>) {
        self.combat_target = target;
    }

    pub fn set_eat_delay(&mut self, ticks: u16) {
        self.eat_delay = ticks;
    }

    pub fn consume_eat_delay(&mut self) -> bool {
        if self.eat_delay > 0 {
            self.eat_delay -= 1;
            true
        } else {
            false
        }
    }

    pub fn queue_retaliate(&mut self, attacker: CombatTarget) {
        if self.auto_retaliate && self.retaliate_target.is_none() {
            self.retaliate_target = Some(attacker);
        }
    }

    async fn flush(&mut self) {
        self.player
            .varp_mut()
            .send_varp(COMBAT_STYLE_VARP, self.combat_style as i32)
            .await;
        self.player
            .varp_mut()
            .send_varp(AUTO_RETALIATE_VARP, !self.auto_retaliate as i32)
            .await;
        self.player
            .varp_mut()
            .send_varp(SPEC_ENERGY_VARP, self.spec_energy as i32)
            .await;
    }
}

#[player_system]
impl PlayerSystem for CombatManager {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            player: ctx.player,
            combat_style: ctx.player_data.combat_style,
            auto_retaliate: ctx.player_data.auto_retaliate,
            spec_energy: ctx.player_data.spec_energy,
            spec_enabled: false,
            spec_dirty: false,
            spec_regen_timer: SPEC_REGEN_TICKS,
            infinite_spec: false,
            combat_target: None,
            eat_delay: 0,
            retaliate_target: None,
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
        Box::pin(async {
            if self.spec_dirty {
                self.spec_dirty = false;
                self.player
                    .varp_mut()
                    .send_varp(SPEC_ENERGY_VARP, self.spec_energy as i32)
                    .await;
                self.player.varp_mut().send_varp(301, self.spec_enabled as i32).await;
            }

            if self.spec_energy < MAX_SPEC_ENERGY {
                self.spec_regen_timer = self.spec_regen_timer.saturating_sub(1);
                if self.spec_regen_timer == 0 {
                    self.spec_energy = (self.spec_energy + SPEC_REGEN_AMOUNT).min(MAX_SPEC_ENERGY);
                    self.spec_regen_timer = SPEC_REGEN_TICKS;
                    self.player
                        .varp_mut()
                        .send_varp(SPEC_ENERGY_VARP, self.spec_energy as i32)
                        .await;
                }
            } else {
                self.spec_regen_timer = SPEC_REGEN_TICKS;
            }

            let Some(target) = self.retaliate_target.take() else {
                return;
            };
            if !self.auto_retaliate {
                return;
            }
            let player = self.player.get_mut();
            if is_action_locked(player) {
                return;
            }
            let world = player.world();
            if world.action_states.lock().contains_key(&player.index) {
                return;
            }

            let alive = match target {
                CombatTarget::Npc(i) => world.npcs.contains(i) && !world.npc(i).is_dying(),
                CombatTarget::Player(i) => world.players.contains(i) && !world.player(i).hitpoints().is_dying(),
            };
            drop(world);
            if !alive {
                return;
            }

            crate::handler::run_action(player, crate::content::start_combat(target));
        })
    }

    fn persist(&self, data: &mut PlayerData) {
        data.combat_style = self.combat_style;
        data.auto_retaliate = self.auto_retaliate;
        data.spec_energy = self.spec_energy;
    }
}
