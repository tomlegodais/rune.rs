use std::sync::Arc;

use macros::player_system;

use crate::{
    player::{
        Clientbound, PlayerSnapshot,
        interface::SubInterface,
        system::{PlayerHandle, PlayerInitContext, PlayerSystem},
    },
    world::World,
};

pub const NPC_BASE: u16 = 63;
pub const PLAYER_BASE: u16 = 240;
pub const OPTIONS_BASE: u16 = 224;
pub const OPTIONS_TITLE: &str = "Select an Option";
pub const OPTIONS_TITLE_COMPONENT: u16 = 1;
pub const OPTIONS_FIRST_COMPONENT: u16 = 2;

pub enum DialogueEntity {
    Npc(u16, Option<u16>),
    Player(Option<u16>),
}

impl DialogueEntity {
    fn base_interface(&self) -> u16 {
        match self {
            Self::Npc(..) => NPC_BASE,
            Self::Player(..) => PLAYER_BASE,
        }
    }

    fn anim(&self) -> Option<u16> {
        match self {
            Self::Npc(_, anim) | Self::Player(anim) => *anim,
        }
    }
}

enum State {
    Idle,
    Active(SubInterface),
    Responded(u8),
}

pub struct Dialogue {
    player: PlayerHandle,
    state: State,
}

impl Dialogue {
    #[rustfmt::skip]
    pub async fn entity_dialogue(&mut self, entity: DialogueEntity, name: &str, text: &str) {
        let lines = word_wrap(text, 55);
        let interface_id = entity.base_interface() + lines.len() as u16;
        let sub = SubInterface::chatbox(interface_id).opaque();
        let anim_id = entity.anim().unwrap_or(9827);
        self.state = State::Active(sub);

        self.player.interface_mut().open_sub(&sub).await;

        match entity {
            DialogueEntity::Npc(npc_id, _) => self.player.if_set_npc_head(interface_id, 2, npc_id).await,
            DialogueEntity::Player(_) => self.player.if_set_player_head(interface_id, 2).await,
        }

        self.player.if_set_anim(interface_id, 2, anim_id).await;
        self.player.interface_mut().set_text(&sub, 3, name).await;

        for (i, line) in lines.iter().enumerate() {
            self.player.interface_mut().set_text(&sub, 4 + i as u16, line).await;
        }
    }

    pub async fn option_dialogue(&mut self, options: &[&str]) {
        let count = options.len().clamp(2, 5);
        let interface_id = OPTIONS_BASE + (count as u16 * 2);
        let sub = SubInterface::chatbox(interface_id).opaque();
        self.state = State::Active(sub);

        self.player.interface_mut().open_sub(&sub).await;
        self.player
            .interface_mut()
            .set_text(&sub, OPTIONS_TITLE_COMPONENT, OPTIONS_TITLE)
            .await;

        for (i, &opt) in options.iter().take(5).enumerate() {
            self.player
                .interface_mut()
                .set_text(&sub, OPTIONS_FIRST_COMPONENT + i as u16, opt)
                .await;
        }
    }

    pub async fn chatbox(&mut self, sub: &SubInterface, texts: &[&str]) {
        let sub = SubInterface::chatbox(sub.interface);
        self.state = State::Active(sub);

        self.player.interface_mut().open_sub(&sub).await;
        for (i, &text) in texts.iter().enumerate() {
            self.player.interface_mut().set_text(&sub, i as u16, text).await;
        }
    }

    pub async fn respond(&mut self, choice: u8) {
        self.close().await;
        self.state = State::Responded(choice);
    }

    pub async fn close(&mut self) {
        if let State::Active(sub) = &self.state {
            self.player.interface_mut().close_sub(sub).await;
        }
        self.state = State::Idle;
    }

    pub fn take_response(&mut self) -> Option<u8> {
        matches!(self.state, State::Responded(_)).then(|| {
            let State::Responded(choice) = std::mem::replace(&mut self.state, State::Idle) else { unreachable!() };
            choice
        })
    }
}

#[player_system]
impl PlayerSystem for Dialogue {
    type TickContext = ();

    fn create(ctx: &PlayerInitContext) -> Self {
        Self {
            player: ctx.player,
            state: State::Idle,
        }
    }

    fn tick_context(_: &Arc<World>, _: &PlayerSnapshot) {}
}

fn word_wrap(text: &str, max_width: usize) -> Vec<String> {
    text.split_whitespace().fold(Vec::<String>::new(), |mut lines, word| {
        match lines.last_mut().filter(|l| l.len() + 1 + word.len() <= max_width) {
            Some(line) => {
                line.push(' ');
                line.push_str(word);
            }
            None => lines.push(word.to_string()),
        }
        lines
    })
}
