use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::Arc,
};

use net::Outbox;
use persistence::player::PlayerData;

use crate::world::World;

pub struct PlayerInitContext {
    pub index: usize,
    pub outbox: Outbox,
    pub player_data: PlayerData,
    pub display_mode: u8,
    pub display_name: String,
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

    fn on_login<'a>(&'a mut self, _ctx: &'a mut SystemContext<'_>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {})
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

type OnLoginFn =
    for<'a> fn(&'a mut dyn Any, &'a mut SystemContext<'_>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

type TickContextFn = fn(&Arc<World>, &super::PlayerSnapshot) -> Box<dyn Any + Send + Sync>;

type TickFn =
    for<'a> fn(&'a mut dyn Any, &'a Box<dyn Any + Send + Sync>) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;

pub struct SystemRegistration {
    pub type_id: fn() -> TypeId,
    pub deps: fn() -> Vec<TypeId>,
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

    unsafe fn take_shared(&self) -> Box<dyn Any + Send + Sync> {
        unsafe { (*self.0.get()).take().expect("system is already taken") }
    }

    unsafe fn put_shared(&self, value: Box<dyn Any + Send + Sync>) {
        let slot = unsafe { &mut *self.0.get() };
        assert!(slot.is_none(), "system slot is not empty");
        *slot = Some(value);
    }
}

struct SystemEntry {
    slot: SystemSlot,
    persist: fn(&dyn Any, &mut PlayerData),
    on_login: OnLoginFn,
    tick_context: TickContextFn,
    tick: TickFn,
}

pub struct SystemStore {
    systems: HashMap<TypeId, SystemEntry>,
    login_order: Vec<TypeId>,
}

impl SystemStore {
    pub fn from_init(ctx: &PlayerInitContext) -> Self {
        let registrations: Vec<&SystemRegistration> = inventory::iter::<SystemRegistration>().collect();

        let login_order = topological_sort(&registrations);

        let mut systems = HashMap::new();
        for reg in &registrations {
            let type_id = (reg.type_id)();
            let value = (reg.factory)(ctx);
            systems.insert(
                type_id,
                SystemEntry {
                    slot: SystemSlot::new(value),
                    persist: reg.persist,
                    on_login: reg.on_login,
                    tick_context: reg.tick_context,
                    tick: reg.tick,
                },
            );
        }

        Self { systems, login_order }
    }

    pub fn get<T: PlayerSystem>(&self) -> &T {
        self.entry::<T>().slot.get_ref().downcast_ref::<T>().unwrap()
    }

    pub fn get_mut<T: PlayerSystem>(&mut self) -> &mut T {
        self.entry_mut::<T>().slot.get_mut().downcast_mut::<T>().unwrap()
    }

    pub(crate) fn guard<T: PlayerSystem>(&self) -> SystemGuard<'_, T> {
        let entry = self
            .systems
            .get(&TypeId::of::<T>())
            .unwrap_or_else(|| panic!("system {} not registered", std::any::type_name::<T>()));

        let value = unsafe { *entry.slot.take_shared().downcast::<T>().unwrap() };

        SystemGuard {
            value: Some(value),
            slot: &entry.slot,
        }
    }

    pub fn for_each_persist(&self, data: &mut PlayerData) {
        for entry in self.systems.values() {
            (entry.persist)(entry.slot.get_ref(), data);
        }
    }

    pub async fn on_login(&mut self, player_info: &mut super::PlayerInfo) {
        for i in 0..self.login_order.len() {
            let type_id = self.login_order[i];
            let entry = self.systems.get_mut(&type_id).unwrap();

            let mut boxed = entry.slot.take_value();
            let on_login = entry.on_login;

            let mut ctx = SystemContext {
                store: &self.systems,
                player_info,
            };

            on_login(boxed.as_mut(), &mut ctx).await;

            self.systems.get_mut(&type_id).unwrap().slot.put_value(boxed);
        }
    }

    pub async fn tick(&mut self, world: &Arc<World>, player: &super::PlayerSnapshot) {
        for i in 0..self.login_order.len() {
            let type_id = self.login_order[i];
            let entry = self.systems.get_mut(&type_id).unwrap();
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

pub struct SystemGuard<'a, T: PlayerSystem> {
    value: Option<T>,
    slot: &'a SystemSlot,
}

impl<T: PlayerSystem> Deref for SystemGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.value.as_ref().unwrap()
    }
}

impl<T: PlayerSystem> DerefMut for SystemGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value.as_mut().unwrap()
    }
}

impl<T: PlayerSystem> Drop for SystemGuard<'_, T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            unsafe { self.slot.put_shared(Box::new(value)) };
        }
    }
}

pub struct SystemContext<'a> {
    store: &'a HashMap<TypeId, SystemEntry>,
    pub player_info: &'a mut super::PlayerInfo,
}

impl<'a> SystemContext<'a> {
    pub fn take<T: PlayerSystem>(&self) -> T {
        let boxed = unsafe {
            self.store
                .get(&TypeId::of::<T>())
                .unwrap_or_else(|| panic!("system {} not registered", std::any::type_name::<T>()))
                .slot
                .take_shared()
        };
        *boxed.downcast::<T>().unwrap()
    }

    pub fn put_back<T: PlayerSystem>(&self, system: T) {
        unsafe {
            self.store
                .get(&TypeId::of::<T>())
                .unwrap_or_else(|| panic!("system {} not registered", std::any::type_name::<T>()))
                .slot
                .put_shared(Box::new(system));
        }
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
