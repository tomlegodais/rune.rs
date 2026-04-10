use std::{
    cell::{Cell, RefCell},
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU16, Ordering},
    },
    task::{Context, Poll, Waker},
};

use crate::npc::Npc;

pub struct NpcActionShared {
    pub delay_remaining: AtomicU16,
}

impl NpcActionShared {
    pub fn new() -> Self {
        Self {
            delay_remaining: AtomicU16::new(0),
        }
    }
}

pub struct NpcActionState {
    pub active: Pin<Box<dyn Future<Output = ()> + Send>>,
    pub shared: Arc<NpcActionShared>,
}

thread_local! {
    static ACTIVE_NPC: Cell<Option<*mut Npc>> = const { Cell::new(None) };
    static ACTIVE_NPC_SHARED: RefCell<Option<Arc<NpcActionShared>>> = const { RefCell::new(None) };
}

pub fn set_action_context(npc: *mut Npc, shared: Arc<NpcActionShared>) {
    ACTIVE_NPC.set(Some(npc));
    ACTIVE_NPC_SHARED.with(|s| *s.borrow_mut() = Some(shared));
}

pub fn clear_action_context() {
    ACTIVE_NPC.set(None);
    ACTIVE_NPC_SHARED.with(|s| *s.borrow_mut() = None);
}

pub fn active_npc<'a>() -> &'a mut Npc {
    let ptr = ACTIVE_NPC.get().expect("no active npc in action context");
    unsafe { &mut *ptr }
}

pub struct NpcRef;

impl std::ops::Deref for NpcRef {
    type Target = Npc;
    fn deref(&self) -> &Npc {
        active_npc()
    }
}

impl std::ops::DerefMut for NpcRef {
    fn deref_mut(&mut self) -> &mut Npc {
        active_npc()
    }
}

pub fn active_shared() -> Arc<NpcActionShared> {
    ACTIVE_NPC_SHARED.with(|s| s.borrow().clone().expect("no active npc shared in action context"))
}

pub struct DelayFuture {
    shared: Arc<NpcActionShared>,
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

pub fn delay(shared: &Arc<NpcActionShared>, ticks: u16) -> DelayFuture {
    shared.delay_remaining.store(ticks, Ordering::Relaxed);
    DelayFuture {
        shared: shared.clone(),
        started: false,
    }
}

pub fn poll_action(state: &mut NpcActionState) -> Poll<()> {
    let waker = Waker::noop();
    let cx = &mut Context::from_waker(waker);
    state.active.as_mut().poll(cx)
}

pub fn fire_action(npc: &mut Npc, action: Pin<Box<dyn Future<Output = ()> + Send + 'static>>) {
    let world = npc.entity.world();
    world.npc_action_states.lock().remove(&npc.index);
    let shared = Arc::new(NpcActionShared::new());
    set_action_context(npc as *mut Npc, shared.clone());
    let mut state = NpcActionState { active: action, shared };
    let poll_result = poll_action(&mut state);
    clear_action_context();
    if poll_result.is_pending() {
        world.npc_action_states.lock().insert(npc.index, state);
    }
}

pub fn resolve(npc: &mut Npc) {
    let world = npc.entity.world();
    let mut state = world.npc_action_states.lock().remove(&npc.index);
    let Some(ref mut s) = state else {
        return;
    };

    if s.shared.delay_remaining.load(Ordering::Relaxed) > 0 {
        s.shared.delay_remaining.fetch_sub(1, Ordering::Relaxed);
    }

    let shared = s.shared.clone();
    set_action_context(npc as *mut Npc, shared);

    let poll_result = poll_action(s);
    clear_action_context();

    if poll_result.is_pending() {
        world.npc_action_states.lock().insert(npc.index, state.unwrap());
    }
}
