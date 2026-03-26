use crate::player::Player;
use net::{ChatMessage, Encodable};
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

pub struct ActionShared {
    pub(crate) delay_remaining: AtomicU16,
    pub(crate) locked: AtomicBool,
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
    static ACTIVE_PLAYER: Cell<Option<*mut Player>> = Cell::new(None);
    static ACTIVE_SHARED: RefCell<Option<Arc<ActionShared>>> = RefCell::new(None);
}

pub fn set_action_context(player: *mut Player, shared: Arc<ActionShared>) {
    ACTIVE_PLAYER.set(Some(player));
    ACTIVE_SHARED.with(|s| *s.borrow_mut() = Some(shared));
}

pub fn clear_action_context() {
    ACTIVE_PLAYER.set(None);
    ACTIVE_SHARED.with(|s| *s.borrow_mut() = None);
}

// why: only safe when called once per poll — proc macro enforces single `let player = ...` binding
pub fn active_player<'a>() -> &'a mut Player {
    let ptr = ACTIVE_PLAYER.get().expect("no active player in action context");
    unsafe { &mut *ptr }
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
        if self.shared.delay_remaining.load(Ordering::Relaxed) == 0 {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
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
    let mut cx = Context::from_waker(&waker);
    state.active.as_mut().poll(&mut cx)
}

pub fn send_message(player: &mut Player, text: &str) {
    let frame = ChatMessage {
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