use crate::npc::{Npc, NpcSnapshot};
use crate::player::{Player, PlayerSnapshot};
use crate::with_movement;
use crate::world::World;
use net::{InboxExt, IncomingMessage};
use parking_lot::Mutex;
use std::collections::HashMap;

trait TickPhase<E> {
    type Context;

    fn context(&self, world: &World) -> Self::Context;

    async fn execute(&self, world: &World, entity: &mut E, ctx: &Self::Context);
}

struct ProcessMessages;
struct ProcessMovement;
struct Sync;
struct Reset;

impl TickPhase<Player> for ProcessMessages {
    type Context = Mutex<HashMap<usize, Vec<IncomingMessage>>>;

    fn context(&self, world: &World) -> Self::Context {
        let pending = world
            .players
            .write()
            .iter_mut()
            .filter_map(|(key, p)| {
                let messages = p.inbox.try_recv_all();
                (!messages.is_empty()).then_some((key + 1, messages))
            })
            .collect();

        Mutex::new(pending)
    }

    async fn execute(&self, _world: &World, player: &mut Player, ctx: &Self::Context) {
        let messages = ctx.lock().remove(&player.index).unwrap_or_default();
        for msg in messages {
            crate::handler::handle(player, msg).await;
        }
    }
}

impl TickPhase<Player> for ProcessMovement {
    type Context = ();

    fn context(&self, _: &World) -> Self::Context {}

    async fn execute(&self, _world: &World, player: &mut Player, _: &()) {
        with_movement!(player, |m, ctx| m.process(&mut ctx).await);
    }
}

impl TickPhase<Npc> for ProcessMovement {
    type Context = ();

    fn context(&self, _: &World) -> Self::Context {}

    async fn execute(&self, world: &World, npc: &mut Npc, _: &()) {
        npc.process_movement();

        let new_region = npc.entity.position.region_id();
        if new_region != npc.entity.current_region {
            world
                .region_map()
                .update_npc_region(npc.index, npc.entity.current_region, new_region);
            npc.entity.current_region = new_region;
        }
    }
}

struct SyncContext {
    player_snapshots: Vec<PlayerSnapshot>,
    npc_snapshots: Vec<NpcSnapshot>,
}

impl TickPhase<Player> for Sync {
    type Context = SyncContext;

    fn context(&self, world: &World) -> Self::Context {
        SyncContext {
            player_snapshots: world.player_snapshots(),
            npc_snapshots: world.npc_snapshots(),
        }
    }

    async fn execute(&self, world: &World, player: &mut Player, ctx: &SyncContext) {
        let new_region = player.entity.position.region_id();
        if new_region != player.entity.current_region {
            world.region_map().update_player_region(
                player.index,
                player.entity.current_region,
                new_region,
            );
            player.entity.current_region = new_region;
        }

        player.tick(&ctx.player_snapshots, &ctx.npc_snapshots).await;
        player.send_info(&ctx.npc_snapshots).await;
    }
}

impl TickPhase<Player> for Reset {
    type Context = ();

    fn context(&self, _: &World) -> Self::Context {}

    async fn execute(&self, _world: &World, player: &mut Player, _: &()) {
        player.reset();
    }
}

impl TickPhase<Npc> for Reset {
    type Context = ();

    fn context(&self, _: &World) -> Self::Context {}

    async fn execute(&self, _world: &World, npc: &mut Npc, _: &()) {
        npc.reset();
    }
}

impl World {
    pub async fn tick(&self) {
        self.run_player_phase(&ProcessMessages).await;
        self.run_player_phase(&ProcessMovement).await;
        self.run_npc_phase(&ProcessMovement).await;
        self.run_player_phase(&Sync).await;
        self.run_player_phase(&Reset).await;
        self.run_npc_phase(&Reset).await;
    }

    async fn run_player_phase(&self, phase: &impl TickPhase<Player>) {
        let ctx = phase.context(self);
        for index in self.players.keys() {
            let mut player = self.players.get_mut(index);
            phase.execute(self, &mut player, &ctx).await;
        }
    }

    async fn run_npc_phase(&self, phase: &impl TickPhase<Npc>) {
        let ctx = phase.context(self);
        for index in self.npcs.keys() {
            let mut npc = self.npcs.get_mut(index);
            phase.execute(self, &mut npc, &ctx).await;
        }
    }
}
