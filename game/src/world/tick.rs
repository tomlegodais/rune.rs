use crate::player::{Player, PlayerSnapshot};
use crate::world::{RegionMap, World};
use net::InboxExt;

trait TickPhase {
    type Context;

    fn context(&self, world: &World) -> Self::Context;

    async fn execute(&self, player: &mut Player, ctx: &Self::Context, region_map: &mut RegionMap);
}

struct ProcessMessages;
struct ProcessMovement;
struct Sync;
struct SendInfo;
struct Reset;

impl TickPhase for ProcessMessages {
    type Context = ();

    fn context(&self, _: &World) -> Self::Context {}

    async fn execute(&self, player: &mut Player, _: &(), _: &mut RegionMap) {
        for message in player.inbox.try_recv_all() {
            crate::handler::handle(player, message).await;
        }
    }
}

impl TickPhase for ProcessMovement {
    type Context = ();

    fn context(&self, _: &World) -> Self::Context {}

    async fn execute(&self, player: &mut Player, _: &(), _: &mut RegionMap) {
        player.process_movement();
    }
}

impl TickPhase for Sync {
    type Context = Vec<PlayerSnapshot>;

    fn context(&self, world: &World) -> Self::Context {
        world.player_snapshots()
    }

    async fn execute(
        &self,
        player: &mut Player,
        snapshots: &Vec<PlayerSnapshot>,
        region_map: &mut RegionMap,
    ) {
        let new_region = player.position.region_id();
        if new_region != player.current_region {
            region_map.update_player_region(player.id, player.current_region, new_region);
            player.current_region = new_region;
        }

        player.tick(snapshots).await;
    }
}

impl TickPhase for SendInfo {
    type Context = ();

    fn context(&self, _: &World) -> Self::Context {}

    async fn execute(&self, player: &mut Player, _: &(), _: &mut RegionMap) {
        player.send_player_info().await;
    }
}

impl TickPhase for Reset {
    type Context = ();

    fn context(&self, _: &World) -> Self::Context {}

    async fn execute(&self, player: &mut Player, _: &(), _: &mut RegionMap) {
        player.reset();
    }
}

impl World {
    pub async fn tick(&mut self) {
        self.run_phase(&ProcessMessages).await;
        self.run_phase(&ProcessMovement).await;
        self.run_phase(&Sync).await;
        self.run_phase(&SendInfo).await;
        self.run_phase(&Reset).await;
    }

    async fn run_phase(&mut self, phase: &impl TickPhase) {
        let ctx = phase.context(self);
        for (_, player) in self.players.iter_mut() {
            phase.execute(player, &ctx, &mut self.region_map).await;
        }
    }
}
