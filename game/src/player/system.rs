use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::Arc,
};

use persistence::PlayerData;

use crate::world::World;

pub struct PlayerInitContext {
    pub index: usize,
    pub player_data: PlayerData,
    pub display_mode: u8,
    pub display_name: String,
    pub player: PlayerHandle,
}

#[derive(Clone, Copy)]
pub struct PlayerHandle(*mut super::Player);

unsafe impl Send for PlayerHandle {}
unsafe impl Sync for PlayerHandle {}

impl PlayerHandle {
    pub fn new(player: *mut super::Player) -> Self {
        Self(player)
    }

    #[allow(clippy::mut_from_ref)]
    pub fn get_mut(&self) -> &mut super::Player {
        unsafe { &mut *self.0 }
    }
}

impl std::ops::Deref for PlayerHandle {
    type Target = super::Player;
    fn deref(&self) -> &super::Player {
        unsafe { &*self.0 }
    }
}

impl std::ops::DerefMut for PlayerHandle {
    fn deref_mut(&mut self) -> &mut super::Player {
        unsafe { &mut *self.0 }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TickPhase {
    Movement,
    Default,
}

pub trait PlayerSystem: Any + Send + Sync + 'static {
    type TickContext: Send + Sync + 'static
    where
        Self: Sized;

    fn create(ctx: &PlayerInitContext) -> Self
    where
        Self: Sized;

    fn dependencies() -> Vec<TypeId>
    where
        Self: Sized,
    {
        vec![]
    }

    fn on_login<'a>(&'a mut self, _player: &'a mut super::Player) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
    }

    fn tick_phase() -> TickPhase
    where
        Self: Sized,
    {
        TickPhase::Default
    }

    fn tick_context(_world: &Arc<World>, _player: &super::PlayerSnapshot) -> Self::TickContext
    where
        Self: Sized;

    fn tick<'a>(&'a mut self, _ctx: &'a Self::TickContext) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>
    where
        Self: Sized,
    {
        Box::pin(async {})
    }

    fn persist(&self, _data: &mut PlayerData) {}
}

type OnLoginFn = for<'a> fn(&'a mut dyn Any, &'a mut super::Player) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

type TickContextFn = fn(&Arc<World>, &super::PlayerSnapshot) -> Box<dyn Any + Send + Sync>;

type TickFn =
    for<'a> fn(&'a mut dyn Any, &'a Box<dyn Any + Send + Sync>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub struct SystemRegistration {
    pub type_id: fn() -> TypeId,
    pub deps: fn() -> Vec<TypeId>,
    pub tick_phase: fn() -> TickPhase,
    pub factory: fn(&PlayerInitContext) -> Box<dyn Any + Send + Sync>,
    pub persist: fn(&dyn Any, &mut PlayerData),
    pub on_login: OnLoginFn,
    pub tick_context: TickContextFn,
    pub tick: TickFn,
}

inventory::collect!(SystemRegistration);

struct SystemSlot(UnsafeCell<Option<Box<dyn Any + Send + Sync>>>);

unsafe impl Sync for SystemSlot {}

impl SystemSlot {
    fn new(value: Box<dyn Any + Send + Sync>) -> Self {
        Self(UnsafeCell::new(Some(value)))
    }

    fn get_ref(&self) -> &dyn Any {
        unsafe { (*self.0.get()).as_ref().expect("system is currently taken") }.as_ref()
    }

    fn get_mut(&mut self) -> &mut dyn Any {
        self.0.get_mut().as_mut().expect("system is currently taken").as_mut()
    }

    fn take_value(&mut self) -> Box<dyn Any + Send + Sync> {
        self.0.get_mut().take().expect("system is already taken")
    }

    fn put_value(&mut self, value: Box<dyn Any + Send + Sync>) {
        let slot = self.0.get_mut();
        assert!(slot.is_none(), "system slot is not empty");
        *slot = Some(value);
    }
}

struct SystemEntry {
    slot: SystemSlot,
    phase: TickPhase,
    persist: fn(&dyn Any, &mut PlayerData),
    on_login: OnLoginFn,
    tick_context: TickContextFn,
    tick: TickFn,
}

pub struct SystemStore {
    systems: HashMap<TypeId, SystemEntry>,
    login_order: Vec<TypeId>,
    tick_order: Vec<TypeId>,
}

impl SystemStore {
    pub fn empty() -> Self {
        Self {
            systems: HashMap::new(),
            login_order: Vec::new(),
            tick_order: Vec::new(),
        }
    }

    pub fn init(&mut self, ctx: &PlayerInitContext) {
        let registrations: Vec<&SystemRegistration> = inventory::iter::<SystemRegistration>().collect();
        self.login_order = topological_sort(&registrations);

        let mut tick_order: Vec<_> = registrations
            .iter()
            .map(|r| ((r.type_id)(), (r.tick_phase)()))
            .collect();

        tick_order.sort_by_key(|(_, phase)| *phase);
        self.tick_order = tick_order.into_iter().map(|(id, _)| id).collect();

        for reg in &registrations {
            let type_id = (reg.type_id)();
            let value = (reg.factory)(ctx);
            self.systems.insert(
                type_id,
                SystemEntry {
                    slot: SystemSlot::new(value),
                    phase: (reg.tick_phase)(),
                    persist: reg.persist,
                    on_login: reg.on_login,
                    tick_context: reg.tick_context,
                    tick: reg.tick,
                },
            );
        }
    }

    pub fn get<T: PlayerSystem>(&self) -> &T {
        self.entry::<T>().slot.get_ref().downcast_ref::<T>().unwrap()
    }

    pub fn get_mut<T: PlayerSystem>(&mut self) -> &mut T {
        self.entry_mut::<T>().slot.get_mut().downcast_mut::<T>().unwrap()
    }

    pub fn for_each_persist(&self, data: &mut PlayerData) {
        for entry in self.systems.values() {
            (entry.persist)(entry.slot.get_ref(), data);
        }
    }

    pub async fn on_login(&mut self, player: &mut super::Player) {
        for i in 0..self.login_order.len() {
            let type_id = self.login_order[i];
            let entry = self.systems.get_mut(&type_id).unwrap();
            let mut boxed = entry.slot.take_value();
            let on_login = entry.on_login;

            on_login(boxed.as_mut(), player).await;

            self.systems.get_mut(&type_id).unwrap().slot.put_value(boxed);
        }
    }

    pub async fn tick_phase(&mut self, phase: TickPhase, world: &Arc<World>, player: &super::PlayerSnapshot) {
        for i in 0..self.tick_order.len() {
            let type_id = self.tick_order[i];
            let entry = self.systems.get_mut(&type_id).unwrap();
            if entry.phase != phase {
                continue;
            }
            let ctx = (entry.tick_context)(world, player);
            let mut boxed = entry.slot.take_value();
            let tick = entry.tick;

            tick(boxed.as_mut(), &ctx).await;
            self.systems.get_mut(&type_id).unwrap().slot.put_value(boxed);
        }
    }

    fn entry<T: PlayerSystem>(&self) -> &SystemEntry {
        self.systems
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("system {} not registered", std::any::type_name::<T>()))
    }

    fn entry_mut<T: PlayerSystem>(&mut self) -> &mut SystemEntry {
        self.systems
            .get_mut(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("system {} not registered", std::any::type_name::<T>()))
    }
}

fn topological_sort(registrations: &[&SystemRegistration]) -> Vec<TypeId> {
    let ids: Vec<TypeId> = registrations.iter().map(|r| (r.type_id)()).collect();
    let deps: HashMap<TypeId, Vec<TypeId>> = registrations.iter().map(|r| ((r.type_id)(), (r.deps)())).collect();
    let mut visited = HashMap::new();
    let mut order = Vec::new();

    for &id in &ids {
        visit(id, &deps, &mut visited, &mut order);
    }

    order
}

fn visit(
    id: TypeId,
    deps: &HashMap<TypeId, Vec<TypeId>>,
    visited: &mut HashMap<TypeId, bool>,
    order: &mut Vec<TypeId>,
) {
    match visited.get(&id) {
        Some(true) => return,
        Some(false) => panic!("circular dependency detected in player systems"),
        None => {}
    }

    visited.insert(id, false);

    if let Some(dep_list) = deps.get(&id) {
        for &dep in dep_list {
            visit(dep, deps, visited, order);
        }
    }

    visited.insert(id, true);
    order.push(id);
}
