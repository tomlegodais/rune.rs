use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU16, Ordering},
    },
    task::{Context, Poll, Waker},
};

use net::{Encodable, MessageGame};

use crate::player::Player;

pub struct ActionShared {
    pub delay_remaining: AtomicU16,
    pub locked: AtomicBool,
}

impl ActionShared {
    pub fn new() -> Self {
        Self {
            delay_remaining: AtomicU16::new(0),
            locked: AtomicBool::new(false),
        }
    }
}

pub struct ActionState {
    pub active: Pin<Box<dyn Future<Output = ()> + Send>>,
    pub shared: Arc<ActionShared>,
}

thread_local! {
    static ACTIVE_PLAYER: Cell<Option<*mut Player>> = const { Cell::new(None) };
    static ACTIVE_SHARED: RefCell<Option<Arc<ActionShared>>> = const { RefCell::new(None) };
}

pub fn set_action_context(player: *mut Player, shared: Arc<ActionShared>) {
    ACTIVE_PLAYER.set(Some(player));
    ACTIVE_SHARED.with(|s| *s.borrow_mut() = Some(shared));
}

pub fn clear_action_context() {
    ACTIVE_PLAYER.set(None);
    ACTIVE_SHARED.with(|s| *s.borrow_mut() = None);
}

pub fn active_player<'a>() -> &'a mut Player {
    let ptr = ACTIVE_PLAYER.get().expect("no active player in action context");
    unsafe { &mut *ptr }
}

pub struct PlayerRef;

impl std::ops::Deref for PlayerRef {
    type Target = Player;
    fn deref(&self) -> &Player {
        active_player()
    }
}

impl std::ops::DerefMut for PlayerRef {
    fn deref_mut(&mut self) -> &mut Player {
        active_player()
    }
}

pub fn active_shared() -> Arc<ActionShared> {
    ACTIVE_SHARED.with(|s| s.borrow().clone().expect("no active shared in action context"))
}

pub struct DelayFuture {
    shared: Arc<ActionShared>,
    started: bool,
}

impl Future for DelayFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if !self.started {
            self.started = true;
            if self.shared.delay_remaining.load(Ordering::Relaxed) > 0 {
                return Poll::Pending;
            }
        }
        if self.shared.delay_remaining.load(Ordering::Relaxed) == 0 { Poll::Ready(()) } else { Poll::Pending }
    }
}

pub struct DialogueFuture;

impl Future for DialogueFuture {
    type Output = u8;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<u8> {
        match active_player().dialogue_mut().take_response() {
            Some(choice) => Poll::Ready(choice),
            None => Poll::Pending,
        }
    }
}

pub fn await_dialogue() -> DialogueFuture {
    DialogueFuture
}

pub fn delay(shared: &Arc<ActionShared>, ticks: u16) -> DelayFuture {
    shared.delay_remaining.store(ticks, Ordering::Relaxed);
    DelayFuture {
        shared: shared.clone(),
        started: false,
    }
}

pub fn poll_action(state: &mut ActionState) -> Poll<()> {
    let waker = Waker::noop();
    let cx = &mut Context::from_waker(waker);
    state.active.as_mut().poll(cx)
}

pub fn send_message(player: &mut Player, text: &str) {
    let frame = MessageGame {
        msg_type: 0,
        text: text.to_string(),
    }
    .encode();
    let _ = player.outbox.try_send(frame);
}

pub fn npc_force_talk(player: &Player, npc_index: usize, text: &str) {
    let world = player.world();
    world.npc_mut(npc_index).force_talk(text.to_string());
}

pub fn lock(shared: &ActionShared) {
    shared.locked.store(true, Ordering::Relaxed);
}

pub fn unlock(shared: &ActionShared) {
    shared.locked.store(false, Ordering::Relaxed);
}

pub struct SeqResetGuard(pub *mut Player);

unsafe impl Send for SeqResetGuard {}

impl Drop for SeqResetGuard {
    fn drop(&mut self) {
        let player = unsafe { &mut *self.0 };
        player.seq(0xFFFF);
        player.spot_anim(0xFFFF);
    }
}

pub fn is_action_locked(player: &Player) -> bool {
    let world = player.world();
    world
        .action_states
        .lock()
        .get(&player.index)
        .is_some_and(|s| s.shared.locked.load(Ordering::Relaxed))
}
