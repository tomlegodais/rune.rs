use std::collections::HashMap;

use net::{InboxExt, IncomingMessage};
use parking_lot::Mutex;

use crate::{
    handler::handle_incoming_message,
    npc::{Npc, NpcSnapshot},
    player::{Player, PlayerSnapshot, resolve_interaction},
    world::World,
};

macro_rules! tick {
    ($world:ident, $( $kind:ident : $phase:ident ),* $(,)?) => {
        $(
            tick!(@step $world, $kind, $phase);
        )*
    };
    (@step $world:ident, player, $phase:ident) => { $world.run_player_phase(&$phase).await; };
    (@step $world:ident, npc,    $phase:ident) => { $world.run_npc_phase(&$phase).await; };
    (@step $world:ident, world,  $phase:ident) => { $world.run_world_phase(&$phase).await; };
}

trait TickPhase<E> {
    type Context;

    fn context(&self, world: &World) -> Self::Context;

    async fn execute(&self, world: &World, entity: &mut E, ctx: &Self::Context);
}

trait WorldTickPhase {
    async fn execute(&self, world: &World);
}

struct ProcessMessages;
struct Tick;
struct Sync;
struct Flush;
struct Reset;
struct WorldTick;

impl TickPhase<Player> for ProcessMessages {
    type Context = Mutex<HashMap<usize, Vec<IncomingMessage>>>;

    fn context(&self, world: &World) -> Self::Context {
        let pending = world
            .players
            .write()
            .iter_mut()
            .filter_map(|(key, p)| {
                let messages = p.get_mut().inbox.try_recv_all();
                (!messages.is_empty()).then_some((key + 1, messages))
            })
            .collect();

        Mutex::new(pending)
    }

    async fn execute(&self, _world: &World, player: &mut Player, ctx: &Self::Context) {
        let messages = ctx.lock().remove(&player.index).unwrap_or_default();
        for msg in messages {
            handle_incoming_message(player, msg).await;
        }
    }
}

impl TickPhase<Player> for Tick {
    type Context = ();

    fn context(&self, _: &World) -> Self::Context {}

    async fn execute(&self, world: &World, player: &mut Player, _: &()) {
        resolve_interaction(player, world);
        player.tick_systems(&world.arc()).await;
    }
}

impl TickPhase<Npc> for Tick {
    type Context = ();

    fn context(&self, _: &World) -> Self::Context {}

    async fn execute(&self, _world: &World, npc: &mut Npc, _: &()) {
        if npc.tick_death() {
            return;
        }
        npc.wander();
        npc.process_movement();
    }
}

impl TickPhase<Player> for Sync {
    type Context = (Vec<PlayerSnapshot>, Vec<NpcSnapshot>);

    fn context(&self, world: &World) -> Self::Context {
        (world.player_snapshots(), world.npc_snapshots())
    }

    async fn execute(&self, world: &World, player: &mut Player, ctx: &(Vec<PlayerSnapshot>, Vec<NpcSnapshot>)) {
        player.sync(&ctx.0, &ctx.1, &world.arc()).await;
    }
}

impl TickPhase<Player> for Flush {
    type Context = Vec<NpcSnapshot>;

    fn context(&self, world: &World) -> Self::Context {
        world.npc_snapshots()
    }

    async fn execute(&self, _world: &World, player: &mut Player, npc_snapshots: &Vec<NpcSnapshot>) {
        player.player_info.flush().await;
        player.npc_info.flush(npc_snapshots, player.position).await;
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

impl WorldTickPhase for WorldTick {
    async fn execute(&self, world: &World) {
        world.decay_obj_stacks().await;
        world.respawn_locs().await;
        world.process_npc_deaths();
        world.tick_npc_respawns();
    }
}

impl World {
    pub async fn tick(&self) {
        tick!(self,
            player: ProcessMessages,
            npc:    Tick,
            player: Tick,
            world:  WorldTick,
            player: Sync,
            player: Flush,
            player: Reset,
            npc:    Reset,
        );
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

    async fn run_world_phase(&self, phase: &impl WorldTickPhase) {
        phase.execute(self).await;
    }
}
