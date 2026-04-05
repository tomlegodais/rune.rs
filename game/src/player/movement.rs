use std::{any::TypeId, future::Future, pin::Pin};

use macros::player_system;
use persistence::player::PlayerData;

use crate::{
    entity::{MoveStep, WalkTarget},
    player::{
        Clientbound, FaceDirectionMask, MoveTypeMask, PlayerSnapshot, TempMoveTypeMask, VarpManager,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
    },
    world::{Direction, Position, Teleport, World, running_direction},
};

pub struct Movement {
    player: PlayerHandle,
    pub running: bool,
    run_energy: u16,
}

impl Movement {
    pub fn run_energy(&self) -> u16 {
        self.run_energy
    }

    pub async fn teleport(&mut self, destination: Position) {
        self.stop().await;
        let player = self.player.get_mut();
        player.player_info.teleport(Teleport {
            from: player.entity.position,
            to: destination,
        });
        player.player_info.add_mask(TempMoveTypeMask::Teleport);
        player.player_info.add_mask(FaceDirectionMask(Direction::South));
        player.entity.position = destination;
        player.entity.face_direction = Direction::South;
    }

    pub async fn walk_to(&mut self, dest: Position, force_run: bool, target: Option<WalkTarget>) {
        if force_run && !self.running {
            self.set_run(true).await;
        }

        let player = self.player.get_mut();
        player.entity.walk_to(dest, target);
        match player.entity.walk_queue.back().copied() {
            Some(end) => self.set_minimap_flag(end).await,
            None => self.reset_minimap_flag().await,
        }
    }

    pub async fn toggle_run(&mut self) {
        self.set_run(!self.running).await;
    }

    pub async fn set_run(&mut self, enabled: bool) {
        self.running = enabled;
        self.player.varp_mut().send_varp(173, if enabled { 1 } else { 0 }).await;
        self.player.player_info.add_mask(MoveTypeMask(self.running));
    }

    async fn process(&mut self) {
        let agility_level = self.player.stat().level(crate::player::Stat::Agility);

        if !self.player.entity.has_steps() {
            self.restore_energy(agility_level).await;
            return;
        }

        let Some(walk_dir) = self.player.entity.step() else {
            return self.stop().await;
        };

        if self.try_run_step(walk_dir).await {
            return;
        }

        self.player.player_info.set_move_step(MoveStep::Walk(walk_dir));
        self.player.player_info.add_mask(TempMoveTypeMask::Walk);
    }

    pub async fn stop(&mut self) {
        self.player.entity.stop();
        self.reset_minimap_flag().await;
    }

    async fn try_run_step(&mut self, walk_dir: Direction) -> bool {
        if !self.running || self.run_energy == 0 {
            return false;
        }

        let Some(run_dir) = self.player.entity.peek_run_step() else { return false };
        let Some(opcode) = running_direction(walk_dir, run_dir) else { return false };

        self.player.entity.commit_run_step(run_dir);
        self.player.player_info.set_move_step(MoveStep::Run(opcode));
        self.player.player_info.add_mask(TempMoveTypeMask::Run);

        self.drain_energy().await;
        true
    }

    async fn drain_energy(&mut self) {
        let agility_level = self.player.stat().level(crate::player::Stat::Agility);
        let prev = self.run_energy / 100;
        self.run_energy = self.run_energy.saturating_sub(self.drain_rate(agility_level));

        if self.run_energy / 100 != prev {
            self.send_run_energy().await;
        }

        if self.run_energy == 0 {
            self.set_run(false).await;
        }
    }

    async fn restore_energy(&mut self, agility: u8) {
        if self.run_energy >= 10_000 {
            return;
        }

        let prev = self.run_energy / 100;
        self.run_energy = (self.run_energy + self.restore_rate(agility)).min(10_000);

        if self.run_energy / 100 != prev {
            self.send_run_energy().await;
        }
    }

    fn drain_rate(&self, agility: u8) -> u16 {
        let weight = self.weight().clamp(0, 64) as f64;
        ((60.0 + 67.0 * weight / 64.0) * (1.0 - agility as f64 / 300.0)) as u16
    }

    fn restore_rate(&self, agility: u8) -> u16 {
        agility as u16 / 10 + 15
    }

    fn weight(&self) -> i32 {
        let grams: i32 = self
            .player
            .worn()
            .slots()
            .0
            .iter()
            .flatten()
            .filter_map(|obj| crate::provider::get_obj_type(obj.id as u32))
            .map(|t| t.weight)
            .sum();
        grams / 1000
    }

    async fn send_run_energy(&mut self) {
        self.player.update_run_energy((self.run_energy / 100) as u8).await;
    }

    async fn set_minimap_flag(&mut self, dest: Position) {
        let region_base = self.player.viewport.region_base;
        let x = (dest.x - region_base.x) as u8;
        let y = (dest.y - region_base.y) as u8;
        self.player.minimap_flag(x, y).await;
    }

    async fn reset_minimap_flag(&mut self) {
        self.player.reset_minimap_flag().await;
    }
}

#[player_system]
impl PlayerSystem for Movement {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            player: ctx.player,
            running: ctx.player_data.running,
            run_energy: ctx.player_data.run_energy,
        }
    }

    fn dependencies() -> Vec<TypeId> {
        vec![TypeId::of::<VarpManager>()]
    }

    fn tick_phase() -> crate::player::system::TickPhase {
        crate::player::system::TickPhase::Movement
    }

    fn on_login<'a>(
        &'a mut self,
        _player: &'a mut crate::player::Player,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            self.player
                .varp_mut()
                .send_varp(173, if self.running { 1 } else { 0 })
                .await;
            self.send_run_energy().await;
        })
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}

    fn tick<'a>(&'a mut self, _ctx: &'a ()) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(self.process())
    }

    fn persist(&self, data: &mut PlayerData) {
        data.running = self.running;
        data.run_energy = self.run_energy();
    }
}
