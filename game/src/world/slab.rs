use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use slab::Slab;

pub struct WorldSlab<T> {
    inner: RwLock<Slab<RwLock<T>>>,
}

impl<T> WorldSlab<T> {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Slab::new()),
        }
    }

    pub fn get(&self, index: usize) -> SlabReadGuard<'_, T> {
        let slab = self.inner.read();
        let entry = slab.get(index - 1).expect("entity not found");
        let guard = unsafe { &*(entry as *const RwLock<T>) }.read();
        SlabReadGuard { _slab: slab, guard }
    }

    pub fn get_mut(&self, index: usize) -> SlabWriteGuard<'_, T> {
        let slab = self.inner.read();
        let entry = slab.get(index - 1).expect("entity not found");
        let guard = unsafe { &*(entry as *const RwLock<T>) }.write();
        SlabWriteGuard { _slab: slab, guard }
    }

    pub fn contains(&self, index: usize) -> bool {
        self.inner.read().contains(index - 1)
    }

    pub fn vacant_index(&self) -> usize {
        self.inner.read().vacant_key() + 1
    }

    pub fn insert(&self, value: T) -> usize {
        self.inner.write().insert(RwLock::new(value)) + 1
    }

    pub fn remove(&self, index: usize) -> T {
        self.inner.write().remove(index - 1).into_inner()
    }

    pub fn keys(&self) -> Vec<usize> {
        self.inner.read().iter().map(|(k, _)| k + 1).collect()
    }

    pub fn map<R>(&self, mut f: impl FnMut(&T) -> R) -> Vec<R> {
        let slab = self.inner.read();
        slab.iter().map(|(_, v)| f(&v.read())).collect()
    }

    pub fn any(&self, mut f: impl FnMut(&T) -> bool) -> bool {
        let slab = self.inner.read();
        slab.iter().any(|(_, v)| f(&v.read()))
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, Slab<RwLock<T>>> {
        self.inner.write()
    }
}

impl<T> Default for WorldSlab<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SlabReadGuard<'a, T> {
    _slab: RwLockReadGuard<'a, Slab<RwLock<T>>>,
    guard: RwLockReadGuard<'a, T>,
}

impl<T> std::ops::Deref for SlabReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.guard
    }
}

pub struct SlabWriteGuard<'a, T> {
    _slab: RwLockReadGuard<'a, Slab<RwLock<T>>>,
    guard: RwLockWriteGuard<'a, T>,
}

impl<T> std::ops::Deref for SlabWriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.guard
    }
}

impl<T> std::ops::DerefMut for SlabWriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.guard
    }
}
