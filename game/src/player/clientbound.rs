use net::{
    IfCloseSub, IfOpenSub, IfOpenTop, IfSetAnim, IfSetEvents, IfSetNpcHead, IfSetPlayerHead, IfSetText, InvEntry,
    InvType, LocAddChange, LocDel, Logout, MapProjAnim, MessageGame, MidiJingle, MinimapToggle, ObjAdd, ObjDel,
    OutboxExt, RunClientScript, ScriptArg, SetPlayerOp, UpdateInvFull, UpdateRunEnergy, UpdateStat, VarbitLarge,
    VarbitSmall, VarcLarge, VarcSmall, VarpLarge, VarpSmall, ZoneFrame,
};

use super::Player;

#[rustfmt::skip]
#[allow(async_fn_in_trait)]
pub trait Clientbound {
    async fn send_message(&mut self, text: impl Into<String> + Send);
    async fn logout(&mut self);
    async fn play_jingle(&mut self, id: u16);

    async fn if_open_top(&mut self, interface: u16, sub: u8);
    async fn if_open_sub(&mut self, parent: u16, component: u16, interface: u16, transparent: bool);
    async fn if_close_sub(&mut self, parent: u16, component: u16);
    async fn if_set_text(&mut self, parent: u16, component: u16, text: impl Into<String> + Send);
    async fn if_set_anim(&mut self, interface_id: u16, component: u16, anim_id: u16);
    async fn if_set_npc_head(&mut self, interface_id: u16, component: u16, npc_id: u16);
    async fn if_set_player_head(&mut self, interface_id: u16, component: u16);
    async fn if_set_events(&mut self, events: IfSetEvents);
    async fn run_client_script(&mut self, id: u32, args: Vec<ScriptArg>);
    async fn set_items_options(&mut self, interface: u16, component: u16, inv_key: u16, width: i32, height: i32, options: &[&str]);

    async fn varp_small(&mut self, id: u16, value: u8);
    async fn varp_large(&mut self, id: u16, value: u32);
    async fn varbit_small(&mut self, id: u16, value: u8);
    async fn varbit_large(&mut self, id: u16, value: u32);
    async fn varc_small(&mut self, id: u16, value: u8);
    async fn varc_large(&mut self, id: u16, value: u32);

    async fn update_inv(&mut self, inv_type: InvType, negative_key: bool, objs: Vec<Option<InvEntry>>);
    async fn update_stat(&mut self, id: u8, level: u8, xp: u32);
    async fn update_run_energy(&mut self, energy: u8);

    async fn minimap_flag(&mut self, x: u8, y: u8);
    async fn reset_minimap_flag(&mut self);

    async fn set_player_op(&mut self, slot: u8, top: bool, op: impl Into<String> + Send);

    async fn loc_add_change(&mut self, zone_frame: ZoneFrame, loc_id: u16, loc_type: u8, rotation: u8, packed_offset: u8);
    async fn loc_del(&mut self, zone_frame: ZoneFrame, loc_type: u8, rotation: u8, packed_offset: u8);
    async fn obj_add(&mut self, zone_frame: ZoneFrame, obj_id: u16, amount: u32, packed_offset: u8);
    async fn obj_del(&mut self, zone_frame: ZoneFrame, obj_id: u16, packed_offset: u8);
    async fn map_projanim(&mut self, projanim: MapProjAnim);

    async fn rebuild_normal(&mut self, init: bool);
}

#[rustfmt::skip]
impl Clientbound for Player {
    async fn send_message(&mut self, text: impl Into<String> + Send) {
        self.outbox.write(MessageGame { msg_type: 0, text: text.into() }).await;
    }

    async fn logout(&mut self) {
        self.outbox.write(Logout).await;
    }

    async fn play_jingle(&mut self, id: u16) {
        self.outbox.write(MidiJingle { id, delay: 0, volume: 255 }).await;
    }

    async fn if_open_top(&mut self, interface: u16, sub: u8) {
        self.outbox.write(IfOpenTop { interface, sub }).await;
    }

    async fn if_open_sub(&mut self, parent: u16, component: u16, interface: u16, transparent: bool) {
        self.outbox.write(IfOpenSub { parent, component, interface, transparent }).await;
    }

    async fn if_close_sub(&mut self, parent: u16, component: u16) {
        self.outbox.write(IfCloseSub { parent, component }).await;
    }

    async fn if_set_text(&mut self, parent: u16, component: u16, text: impl Into<String> + Send) {
        self.outbox.write(IfSetText { parent, component, text: text.into() }).await;
    }

    async fn if_set_anim(&mut self, interface_id: u16, component: u16, anim_id: u16) {
        self.outbox.write(IfSetAnim { interface_id, component, anim_id }).await;
    }

    async fn if_set_npc_head(&mut self, interface_id: u16, component: u16, npc_id: u16) {
        self.outbox.write(IfSetNpcHead { interface_id, component, npc_id }).await;
    }

    async fn if_set_player_head(&mut self, interface_id: u16, component: u16) {
        self.outbox.write(IfSetPlayerHead { interface_id, component }).await;
    }

    async fn if_set_events(&mut self, events: IfSetEvents) {
        self.outbox.write(events).await;
    }

    async fn run_client_script(&mut self, id: u32, args: Vec<ScriptArg>) {
        self.outbox.write(RunClientScript { id, args }).await;
    }

    async fn set_items_options(&mut self, interface: u16, component: u16, inv_key: u16, width: i32, height: i32, options: &[&str]) {
        let mut args: Vec<ScriptArg> = options.iter().rev().map(|s| ScriptArg::Str(s.to_string())).collect();
        args.extend([
            ScriptArg::Int(-1),
            ScriptArg::Int(0),
            ScriptArg::Int(height),
            ScriptArg::Int(width),
            ScriptArg::Int(inv_key as i32),
            ScriptArg::Int((interface as i32) << 16 | component as i32),
        ]);
        self.outbox.write(RunClientScript { id: 150, args }).await;
    }

    async fn varp_small(&mut self, id: u16, value: u8) {
        self.outbox.write(VarpSmall { id, value }).await;
    }

    async fn varp_large(&mut self, id: u16, value: u32) {
        self.outbox.write(VarpLarge { id, value }).await;
    }

    async fn varbit_small(&mut self, id: u16, value: u8) {
        self.outbox.write(VarbitSmall { id, value }).await;
    }

    async fn varbit_large(&mut self, id: u16, value: u32) {
        self.outbox.write(VarbitLarge { id, value }).await;
    }

    async fn varc_small(&mut self, id: u16, value: u8) {
        self.outbox.write(VarcSmall { id, value }).await;
    }

    async fn varc_large(&mut self, id: u16, value: u32) {
        self.outbox.write(VarcLarge { id, value }).await;
    }

    async fn update_inv(&mut self, inv_type: InvType, negative_key: bool, objs: Vec<Option<InvEntry>>) {
        self.outbox.write(UpdateInvFull { inv_type, negative_key, objs }).await;
    }

    async fn update_stat(&mut self, id: u8, level: u8, xp: u32) {
        self.outbox.write(UpdateStat { id, level, xp }).await;
    }

    async fn update_run_energy(&mut self, energy: u8) {
        self.outbox.write(UpdateRunEnergy(energy)).await;
    }

    async fn minimap_flag(&mut self, x: u8, y: u8) {
        self.outbox.write(MinimapToggle { x, y }).await;
    }

    async fn reset_minimap_flag(&mut self) {
        self.outbox.write(MinimapToggle::reset()).await;
    }

    async fn set_player_op(&mut self, slot: u8, top: bool, op: impl Into<String> + Send) {
        self.outbox.write(SetPlayerOp { slot, top, op: op.into() }).await;
    }

    async fn loc_add_change(&mut self, zone_frame: ZoneFrame, loc_id: u16, loc_type: u8, rotation: u8, packed_offset: u8) {
        self.outbox.write(LocAddChange { zone_frame, loc_id, loc_type, rotation, packed_offset }).await;
    }

    async fn loc_del(&mut self, zone_frame: ZoneFrame, loc_type: u8, rotation: u8, packed_offset: u8) {
        self.outbox.write(LocDel { zone_frame, loc_type, rotation, packed_offset }).await;
    }

    async fn obj_add(&mut self, zone_frame: ZoneFrame, obj_id: u16, amount: u32, packed_offset: u8) {
        self.outbox.write(ObjAdd { zone_frame, obj_id, amount, packed_offset }).await;
    }

    async fn obj_del(&mut self, zone_frame: ZoneFrame, obj_id: u16, packed_offset: u8) {
        self.outbox.write(ObjDel { zone_frame, obj_id, packed_offset }).await;
    }

    async fn map_projanim(&mut self, projanim: MapProjAnim) {
        self.outbox.write(projanim).await;
    }

    async fn rebuild_normal(&mut self, init: bool) {
        self.outbox.write(net::RebuildNormal {
            init,
            position_bits: self.position.to_bits(),
            player_index: self.index,
            view_distance: self.viewport.view_distance,
            chunk_x: self.position.chunk_x(),
            chunk_y: self.position.chunk_y(),
            region_count: self.viewport.region_ids().len(),
            region_hashes: std::array::from_fn(|i| self.player_info[i].region_hash),
        }).await;
    }
}
