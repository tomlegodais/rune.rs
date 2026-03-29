use crate::entity::Entity;
use crate::entity::MoveStep;
use crate::player::system::{PlayerInitContext, PlayerSystem, SystemContext};
use crate::player::{
    FaceDirectionMask, MoveTypeMask, PlayerInfo, PlayerSnapshot, TempMoveTypeMask, VarpManager,
};
use crate::world::{Direction, Position, Teleport, World, running_direction};
use macros::player_system;
use net::{MinimapFlag, Outbox, OutboxExt, RunEnergy};
use persistence::player::PlayerData;
use std::any::TypeId;
use std::future::Future;
use std::pin::Pin;

pub struct Movement {
    outbox: Outbox,
    pub running: bool,
    run_energy: u16,
}

pub struct MovementContext<'a> {
    pub entity: &'a mut Entity,
    pub player_info: &'a mut PlayerInfo,
    pub varps: &'a mut VarpManager,
    pub agility_level: u8,
    pub region_base: Position,
}

impl Movement {
    pub fn new(outbox: Outbox, running: bool, run_energy: u16) -> Self {
        Self {
            outbox,
            running,
            run_energy,
        }
    }

    pub fn run_energy(&self) -> u16 {
        self.run_energy
    }

    pub async fn teleport(&mut self, ctx: &mut MovementContext<'_>, destination: Position) {
        self.stop(ctx).await;
        ctx.player_info.teleport(Teleport {
            from: ctx.entity.position,
            to: destination,
        });
        ctx.player_info.add_mask(TempMoveTypeMask::Teleport);
        ctx.player_info
            .add_mask(FaceDirectionMask(Direction::South));
        ctx.entity.position = destination;
        ctx.entity.face_direction = Direction::South;
    }

    pub async fn walk_to(
        &mut self,
        ctx: &mut MovementContext<'_>,
        dest: Position,
        force_run: bool,
        target: Option<(i32, i32, u8)>,
    ) {
        if force_run && !self.running {
            self.set_run(ctx, true).await;
        }

        ctx.entity.walk_to(dest, target);
        match ctx.entity.walk_queue.back().copied() {
            Some(end) => self.set_minimap_flag(end, ctx.region_base).await,
            None => self.reset_minimap_flag().await,
        }
    }

    pub async fn set_run(&mut self, ctx: &mut MovementContext<'_>, enabled: bool) {
        self.running = enabled;
        ctx.varps.send_varp(173, if enabled { 1 } else { 0 }).await;
        ctx.player_info.add_mask(MoveTypeMask(self.running));
    }

    #[rustfmt::skip]
    pub async fn process(&mut self, ctx: &mut MovementContext<'_>) {
        if !ctx.entity.has_steps() {
            self.restore_energy(ctx.agility_level).await;
            return;
        }

        let Some(walk_dir) = ctx.entity.step() else {
            return self.stop(ctx).await;
        };

        if self.try_run_step(ctx, walk_dir).await {
            return;
        }

        ctx.player_info.set_move_step(MoveStep::Walk(walk_dir));
        ctx.player_info.add_mask(TempMoveTypeMask::Walk);
    }

    pub async fn stop(&mut self, ctx: &mut MovementContext<'_>) {
        ctx.entity.stop();
        self.reset_minimap_flag().await;
    }

    #[rustfmt::skip]
    async fn try_run_step(&mut self, ctx: &mut MovementContext<'_>, walk_dir: Direction) -> bool {
        if !self.running || self.run_energy == 0 {
            return false;
        }

        let Some(run_dir) = ctx.entity.peek_run_step() else { return false };
        let Some(opcode) = running_direction(walk_dir, run_dir) else { return false };

        ctx.entity.commit_run_step(run_dir);
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

    fn drain_rate(&self, agility: u8) -> u16 {
        let weight = self.weight().clamp(0, 64);
        let base = 60 + (67 * weight / 64);
        (base as f64 * (1.0 - agility as f64 / 300.0)) as u16
    }

    fn restore_rate(&self, agility: u8) -> u16 {
        agility as u16 / 10 + 15
    }

    fn weight(&self) -> i32 {
        0
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
}

#[player_system]
impl PlayerSystem for Movement {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self::new(ctx.outbox.clone(), ctx.player_data.running, ctx.player_data.run_energy)
    }

    fn dependencies() -> Vec<TypeId> {
        vec![TypeId::of::<VarpManager>()]
    }

    fn on_login<'a>(
        &'a mut self,
        ctx: &'a mut SystemContext<'_>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            let mut varps = ctx.take::<VarpManager>();
            varps.send_varp(173, if self.running { 1 } else { 0 }).await;
            ctx.put_back(varps);
            self.send_run_energy().await;
        })
    }

    fn tick_context(_: &std::sync::Arc<World>, _: &PlayerSnapshot) {}

    fn persist(&self, data: &mut PlayerData) {
        data.running = self.running;
        data.run_energy = self.run_energy();
    }
}
