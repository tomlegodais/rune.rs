use crate::{Encodable, Frame, Prefix};
use tokio_util::bytes::{BufMut, BytesMut};
use util::BytesMutExt;

#[derive(Default)]
pub struct IfEvents(u32);

impl IfEvents {
    pub fn standard_click(mut self, allowed: bool) -> Self {
        self.0 = (self.0 & !0x1) | (allowed as u32);
        self
    }

    pub fn right_click(mut self, option: u32, allowed: bool) -> Self {
        assert!(option <= 9, "option must be 0-9");
        self.0 = (self.0 & !(0x1 << (option + 1))) | ((allowed as u32) << (option + 1));
        self
    }

    pub fn use_on_ground_items(mut self, allow: bool) -> Self {
        self.0 = (self.0 & !(1 << 11)) | ((allow as u32) << 11);
        self
    }

    pub fn use_on_npcs(mut self, allow: bool) -> Self {
        self.0 = (self.0 & !(1 << 12)) | ((allow as u32) << 12);
        self
    }

    pub fn use_on_objects(mut self, allow: bool) -> Self {
        self.0 = (self.0 & !(1 << 13)) | ((allow as u32) << 13);
        self
    }

    pub fn use_on_players(mut self, allow: bool) -> Self {
        self.0 = (self.0 & !(1 << 14)) | ((allow as u32) << 14);
        self
    }

    pub fn use_on_self(mut self, allow: bool) -> Self {
        self.0 = (self.0 & !(1 << 15)) | ((allow as u32) << 15);
        self
    }

    pub fn use_on_components(mut self, allow: bool) -> Self {
        self.0 = (self.0 & !(1 << 16)) | ((allow as u32) << 16);
        self
    }

    pub fn depth(mut self, depth: u32) -> Self {
        assert!(depth <= 7, "depth must be 0-7");
        self.0 = (self.0 & !(0x7 << 18)) | (depth << 18);
        self
    }

    pub fn can_drag_onto(mut self, allow: bool) -> Self {
        self.0 = (self.0 & !(1 << 21)) | ((allow as u32) << 21);
        self
    }

    pub fn can_use_on(mut self, allow: bool) -> Self {
        self.0 = (self.0 & !(1 << 22)) | ((allow as u32) << 22);
        self
    }

    pub fn can_drag(mut self, allow: bool) -> Self {
        self.0 = (self.0 & !(1 << 23)) | ((allow as u32) << 23);
        self
    }
}

impl From<IfEvents> for u32 {
    fn from(e: IfEvents) -> u32 {
        e.0
    }
}

#[macro_export]
macro_rules! if_events {
    (@munch $e:expr; ) => { $e };

    (@munch $e:expr; , $($rest:tt)*) => {
        if_events!(@munch $e; $($rest)*)
    };

    (@munch $e:expr; right_click[$($opt:expr),* $(,)?] $($rest:tt)*) => {
        if_events!(
            @munch
            {
                let mut e = $e;
                $( e = e.right_click($opt, true); )*
                e
            };
            $($rest)*
        )
    };

    (@munch $e:expr; use_on[$($target:ident),* $(,)?] $($rest:tt)*) => {
        if_events!(
            @munch
            {
                let mut e = $e;
                $( e = if_events!(@use_on e, $target); )*
                e
            };
            $($rest)*
        )
    };

    (@munch $e:expr; depth[$d:literal] $($rest:tt)*) => {
        if_events!(@munch $e.depth($d); $($rest)*)
    };

    (@munch $e:expr; can_use_on $($rest:tt)*) => {
        if_events!(@munch $e.can_use_on(true); $($rest)*)
    };

    (@munch $e:expr; can_drag_onto $($rest:tt)*) => {
        if_events!(@munch $e.can_drag_onto(true); $($rest)*)
    };

    (@munch $e:expr; can_drag $($rest:tt)*) => {
        if_events!(@munch $e.can_drag(true); $($rest)*)
    };

    (@use_on $e:ident, ground)      => { $e.use_on_ground_items(true) };
    (@use_on $e:ident, npcs)        => { $e.use_on_npcs(true) };
    (@use_on $e:ident, objects)     => { $e.use_on_objects(true) };
    (@use_on $e:ident, players)     => { $e.use_on_players(true) };
    (@use_on $e:ident, self_player) => { $e.use_on_self(true) };
    (@use_on $e:ident, components)  => { $e.use_on_components(true) };

    (@munch $e:expr; $bad:tt $($rest:tt)*) => {
        compile_error!("invalid if_events! syntax");
    };

    ($($tt:tt)*) => {{
        let __e = if_events!(@munch $crate::IfEvents::default(); $($tt)*);
        u32::from(__e)
    }};
}

#[macro_export]
macro_rules! if_set_events {
    (
        interface_id: $interface_id:expr,
        component_id: $component_id:expr,
        slots: [$from_slot:expr => $to_slot:expr],
        $($events:tt)*
    ) => {
        $crate::IfSetEvents {
            interface_id: $interface_id,
            component_id: $component_id,
            from_slot: $from_slot,
            to_slot: $to_slot,
            events: $crate::if_events!($($events)*),
        }
    };
}

pub struct IfSetEvents {
    pub interface_id: u16,
    pub component_id: u16,
    pub from_slot: u16,
    pub to_slot: u16,
    pub events: u32,
}

impl Encodable for IfSetEvents {
    fn encode(self) -> Frame {
        let mut buf = BytesMut::new();
        buf.put_u16_le_add(self.from_slot);
        buf.put_u32_mid_be(self.events);
        buf.put_u16(0);
        buf.put_u32(((self.interface_id as u32) << 16) | self.component_id as u32);
        buf.put_u16_le(self.to_slot);

        Frame {
            opcode: 75,
            prefix: Prefix::Fixed,
            payload: buf.freeze(),
        }
    }
}
