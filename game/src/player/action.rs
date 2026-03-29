use crate::player::Player;
use net::{ChatMessage, Encodable};
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
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
    let ptr = ACTIVE_PLAYER
        .get()
        .expect("no active player in action context");
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
    ACTIVE_SHARED.with(|s| {
        s.borrow()
            .clone()
            .expect("no active shared in action context")
    })
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
    let cx = &mut Context::from_waker(waker);
    state.active.as_mut().poll(cx)
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

type ActionCallback = Box<dyn FnMut() + Send>;

pub struct SkillActionBuilder {
    shared: Arc<ActionShared>,
    anim: Option<u16>,
    spot_anim: Option<u16>,
    interval: u16,
    on_attempt: Option<ActionCallback>,
    success_predicate: Box<dyn Fn() -> bool + Send>,
    success_handler: ActionCallback,
}

impl SkillActionBuilder {
    pub fn new(shared: Arc<ActionShared>) -> Self {
        Self {
            shared,
            anim: None,
            spot_anim: None,
            interval: 4,
            on_attempt: None,
            success_predicate: Box::new(|| false),
            success_handler: Box::new(|| {}),
        }
    }

    pub fn anim(mut self, id: u16) -> Self {
        self.anim = Some(id);
        self
    }

    pub fn spot_anim(mut self, id: u16) -> Self {
        self.spot_anim = Some(id);
        self
    }

    pub fn interval(mut self, ticks: u16) -> Self {
        self.interval = ticks;
        self
    }

    pub fn on_attempt(mut self, handler: impl FnMut() + Send + 'static) -> Self {
        self.on_attempt = Some(Box::new(handler));
        self
    }

    pub fn on_success(
        mut self,
        predicate: impl Fn() -> bool + Send + 'static,
        handler: impl FnMut() + Send + 'static,
    ) -> Self {
        self.success_predicate = Box::new(predicate);
        self.success_handler = Box::new(handler);
        self
    }
}

impl IntoFuture for SkillActionBuilder {
    type Output = ();
    type IntoFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

    #[rustfmt::skip]
    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let mut on_attempt = self.on_attempt;
            let mut handler = self.success_handler;

            loop {
                let player = active_player();
                self.anim.inspect(|&id| { player.anim(id); });
                self.spot_anim.inspect(|&id| { player.spot_anim(id); });

                if let Some(f) = on_attempt.as_mut() { f(); }

                delay(&self.shared, self.interval).await;

                if (self.success_predicate)() {
                    self.anim.inspect(|_| { player.anim(0xFFFF); });
                    handler();
                    break;
                }
            }
        })
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
