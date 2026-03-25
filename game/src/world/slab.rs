use parking_lot::{
    MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use slab::Slab;

pub struct WorldSlab<T> {
    inner: RwLock<Slab<T>>,
}

impl<T> WorldSlab<T> {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Slab::new()),
        }
    }

    pub fn get(&self, index: usize) -> MappedRwLockReadGuard<'_, T> {
        RwLockReadGuard::map(self.inner.read(), |s| {
            s.get(index - 1).expect("entity not found")
        })
    }

    pub fn get_mut(&self, index: usize) -> MappedRwLockWriteGuard<'_, T> {
        RwLockWriteGuard::map(self.inner.write(), |s| {
            s.get_mut(index - 1).expect("entity not found")
        })
    }

    pub fn contains(&self, index: usize) -> bool {
        self.inner.read().contains(index - 1)
    }

    pub fn vacant_index(&self) -> usize {
        self.inner.read().vacant_key() + 1
    }

    pub fn insert(&self, value: T) -> usize {
        self.inner.write().insert(value) + 1
    }

    pub fn remove(&self, index: usize) -> T {
        self.inner.write().remove(index - 1)
    }

    pub fn keys(&self) -> Vec<usize> {
        self.inner.read().iter().map(|(k, _)| k + 1).collect()
    }

    pub fn for_each(&self, mut f: impl FnMut(usize, &T)) {
        for (key, val) in self.inner.read().iter() {
            f(key + 1, val);
        }
    }

    pub fn for_each_mut(&self, mut f: impl FnMut(usize, &mut T)) {
        for (key, val) in self.inner.write().iter_mut() {
            f(key + 1, val);
        }
    }

    pub fn map<R>(&self, mut f: impl FnMut(&T) -> R) -> Vec<R> {
        self.inner.read().iter().map(|(_, v)| f(v)).collect()
    }

    pub fn any(&self, mut f: impl FnMut(&T) -> bool) -> bool {
        self.inner.read().iter().any(|(_, v)| f(v))
    }

    pub fn write(&self) -> RwLockWriteGuard<'_, Slab<T>> {
        self.inner.write()
    }
}

impl<T> Default for WorldSlab<T> {
    fn default() -> Self {
        Self::new()
    }
}
