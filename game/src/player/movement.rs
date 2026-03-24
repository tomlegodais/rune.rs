use crate::player::state::MoveStep;
use crate::player::{MoveTypeMask, PlayerInfo, TempMoveTypeMask, VarpManager};
use crate::world::{running_direction, Direction, Position};
use net::{MinimapFlag, Outbox, OutboxExt, RunEnergy};
use std::collections::VecDeque;

pub struct Movement {
    outbox: Outbox,
    pub walk_queue: VecDeque<Position>,
    pub running: bool,
    run_energy: u16,
}

pub struct MovementContext<'a> {
    pub position: &'a mut Position,
    pub player_info: &'a mut PlayerInfo,
    pub varps: &'a mut VarpManager,
    pub agility_level: u8,
    pub region_base: Position,
}

impl Movement {
    pub fn new(outbox: Outbox, running: bool, run_energy: u16) -> Self {
        Self {
            outbox,
            walk_queue: VecDeque::new(),
            running,
            run_energy,
        }
    }

    pub async fn on_login(&mut self, varps: &mut VarpManager) {
        varps.send_varp(173, if self.running { 1 } else { 0 }).await;
        self.send_run_energy().await;
    }

    pub async fn set_run(&mut self, ctx: &mut MovementContext<'_>, enabled: bool) {
        self.running = enabled;
        ctx.varps.send_varp(173, if enabled { 1 } else { 0 }).await;
        ctx.player_info.add_mask(MoveTypeMask(self.running));
    }

    pub async fn walk_to(
        &mut self,
        ctx: &mut MovementContext<'_>,
        dest: Position,
        force_run: bool,
    ) {
        if force_run && !self.running {
            self.set_run(ctx, true).await;
        }

        self.walk_queue = crate::world::find_path(*ctx.position, dest);
        match self.walk_queue.back().copied() {
            Some(end) => self.set_minimap_flag(end, ctx.region_base).await,
            None => self.reset_minimap_flag().await,
        }
    }

    #[rustfmt::skip]
    pub async fn process(&mut self, ctx: &mut MovementContext<'_>) {
        if self.walk_queue.is_empty() {
            self.restore_energy(ctx.agility_level).await;
            return;
        }

        let Some(next) = self.walk_queue.pop_front() else { return };
        let Some(walk_dir) = ctx.position.direction_to(next) else {
            return self.stop().await;
        };

        if !crate::world::Collision::can_move(*ctx.position, walk_dir) {
            return self.stop().await;
        }

        *ctx.position = next;

        if self.try_run_step(ctx, walk_dir).await {
            return;
        }

        ctx.player_info.set_move_step(MoveStep::Walk(walk_dir));
        ctx.player_info.add_mask(TempMoveTypeMask::Walk);
    }

    #[rustfmt::skip]
    async fn try_run_step(&mut self, ctx: &mut MovementContext<'_>, walk_dir: Direction) -> bool {
        if !self.running || self.run_energy == 0 {
            return false;
        }

        let Some(&run_pos) = self.walk_queue.front() else { return false };
        let Some(run_dir) = ctx.position.direction_to(run_pos) else { return false };

        if !crate::world::Collision::can_move(*ctx.position, run_dir) {
            return false;
        }

        let Some(opcode) = running_direction(walk_dir, run_dir) else { return false };

        self.walk_queue.pop_front();
        *ctx.position = run_pos;
        ctx.player_info.set_move_step(MoveStep::Run(opcode));
        ctx.player_info.add_mask(TempMoveTypeMask::Run);

        self.drain_energy(ctx).await;
        true
    }

    async fn drain_energy(&mut self, ctx: &mut MovementContext<'_>) {
        let prev = self.run_energy / 100;
        self.run_energy = self
            .run_energy
            .saturating_sub(self.drain_rate(ctx.agility_level));

        if self.run_energy / 100 != prev {
            self.send_run_energy().await;
        }

        if self.run_energy == 0 {
            self.set_run(ctx, false).await;
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

    pub async fn stop(&mut self) {
        self.walk_queue.clear();
        self.reset_minimap_flag().await;
    }

    fn drain_rate(&self, agility: u8) -> u16 {
        let weight = self.weight().clamp(0, 64);
        let base = 60 + (67 * weight / 64);
        (base as f64 * (1.0 - agility as f64 / 300.0)) as u16
    }

    fn restore_rate(&self, agility: u8) -> u16 {
        agility as u16 / 10 + 15
    }

    async fn send_run_energy(&mut self) {
        self.outbox
            .write(RunEnergy((self.run_energy / 100) as u8))
            .await;
    }

    async fn set_minimap_flag(&mut self, dest: Position, region_base: Position) {
        let x = (dest.x - region_base.x) as u8;
        let y = (dest.y - region_base.y) as u8;
        self.outbox.write(MinimapFlag { x, y }).await;
    }

    async fn reset_minimap_flag(&mut self) {
        self.outbox.write(MinimapFlag::reset()).await;
    }

    fn weight(&self) -> i32 {
        0
    }

    pub fn run_energy(&self) -> u16 {
        self.run_energy
    }
}
