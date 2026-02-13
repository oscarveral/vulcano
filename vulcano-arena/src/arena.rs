//! Generational arena implementation.

use std::{
    fmt::{Debug, Formatter},
    mem::ManuallyDrop,
    ops::{Deref, DerefMut, Index, IndexMut},
};

use crate::Key;

/// Internal slot data: either occupied with a value or pointing to the next.
union Container<T> {
    /// Stored data in the container.
    data: ManuallyDrop<T>,
    /// Index of the next free slot.
    next: usize,
}

/// Slot that can store data and the current version of it.
struct Slot<T> {
    /// Data stored in the slot.
    container: Container<T>,
    /// Current slot version. Even is empty, odd is occupied.
    version: usize,
}

/// Safe access to the slot data.
enum Access<'a, T: 'a> {
    /// Occupied variant with a reference to the stored data.
    Occupied(&'a T),
    /// Empty variant with a reference to next free slot index.
    Empty(&'a usize),
}

impl<T> Slot<T> {
    /// Check if the slot contains data.
    pub fn empty(&self) -> bool {
        self.version & 1 == 0
    }

    /// Get a reference to the contained element or to the index of the next free slot.
    pub fn get(&self) -> Access<'_, T> {
        unsafe {
            if self.empty() {
                Access::Empty(&self.container.next)
            } else {
                Access::Occupied(&self.container.data)
            }
        }
    }
}

impl<T> Drop for Slot<T> {
    fn drop(&mut self) {
        if std::mem::needs_drop::<T>() && !self.empty() {
            unsafe {
                ManuallyDrop::drop(&mut self.container.data);
            }
        }
    }
}

impl<T: Clone> Clone for Slot<T> {
    fn clone(&self) -> Self {
        Self {
            container: unsafe {
                if self.empty() {
                    Container {
                        next: self.container.next,
                    }
                } else {
                    Container {
                        data: self.container.data.clone(),
                    }
                }
            },
            version: self.version,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match (self.empty(), source.empty()) {
            (true, true) => unsafe {
                self.container.next = source.container.next;
            },
            (true, false) => unsafe {
                self.container = Container {
                    data: source.container.data.clone(),
                }
            },
            (false, true) => unsafe {
                ManuallyDrop::drop(&mut self.container.data);
                self.container = Container {
                    next: source.container.next,
                }
            },
            (false, false) => unsafe {
                self.container.data.clone_from(&source.container.data);
            },
        }
        self.version = source.version;
    }
}

impl<T: Debug> Debug for Slot<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.get() {
            Access::Occupied(data) => data.fmt(f),
            Access::Empty(next) => write!(f, "next {}", next),
        }
    }
}

/// Slotmap arena structure.
pub struct Arena<T> {
    /// Storage for the slots.
    slots: Vec<Slot<T>>,
    /// Index of the next free slot.
    head: usize,
    /// Number of occupied slots.
    count: usize,
}

impl<T> Arena<T> {
    /// Create a new arena with the given capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        let slots = Vec::with_capacity(capacity);
        Self {
            slots,
            head: 0,
            count: 0,
        }
    }

    /// Create a new empty arena.
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Returns the number of elements in the arena.
    pub fn len(&self) -> usize {
        self.count
    }

    /// Returns true if the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Returns the capacity of the arena.
    pub fn capacity(&self) -> usize {
        self.slots.capacity()
    }

    /// Reserves capacity for at least additional elements to be inserted in the arena.
    pub fn reserve(&mut self, additional: usize) {
        self.slots.reserve(additional);
    }

    /// Tries to reserve capacity for at least additional elements to be inserted in the arena.
    pub fn try_reserve(
        &mut self,
        additional: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        self.slots.try_reserve(additional)
    }

    /// Returns true if the arena contains the given key.
    pub fn contains_key(&self, key: Key) -> bool {
        self.slots
            .get(key.index())
            .is_some_and(|slot| slot.version == key.version())
    }

    /// Returns a reference to the value corresponding to the key.
    pub fn get(&self, key: Key) -> Option<&T> {
        self.slots
            .get(key.index())
            .filter(|s| s.version == key.version())
            .map(|s| unsafe { s.container.data.deref() })
    }

    /// Returns a mutable reference to the value corresponding to the key.
    pub fn get_mut(&mut self, key: Key) -> Option<&mut T> {
        self.slots
            .get_mut(key.index())
            .filter(|s| s.version == key.version())
            .map(|s| unsafe { s.container.data.deref_mut() })
    }

    /// Insert a value into the arena, returning a key to access it.
    pub fn insert(&mut self, value: T) -> Key {
        let index = if self.head < self.slots.len() {
            let slot = &mut self.slots[self.head];
            let index = self.head;
            self.head = unsafe { slot.container.next };
            slot.container = Container {
                data: ManuallyDrop::new(value),
            };
            slot.version += 1;
            index
        } else {
            let index = self.slots.len();
            self.slots.push(Slot {
                container: Container {
                    data: ManuallyDrop::new(value),
                },
                version: 1,
            });
            self.head = self.slots.len();
            index
        };
        self.count += 1;
        Key {
            index,
            version: self.slots[index].version,
        }
    }

    /// Remove the value associated with the given key, returning it if it exists.
    pub fn remove(&mut self, key: Key) -> Option<T> {
        let slot = self.slots.get_mut(key.index())?;
        if slot.version != key.version() {
            return None;
        }
        let value = unsafe { ManuallyDrop::take(&mut slot.container.data) };
        slot.container = Container { next: self.head };
        slot.version += 1;
        self.head = key.index();
        self.count -= 1;
        Some(value)
    }

    /// Insert a value created from a closure that receives the key it will be stored under.
    pub fn insert_with_key(&mut self, f: impl FnOnce(Key) -> T) -> Key {
        let (index, version) = if self.head < self.slots.len() {
            let slot = &self.slots[self.head];
            (self.head, slot.version + 1)
        } else {
            (self.slots.len(), 1)
        };
        let key = Key { index, version };
        self.insert(f(key))
    }
}

impl<T> Index<Key> for Arena<T> {
    type Output = T;

    fn index(&self, key: Key) -> &Self::Output {
        self.get(key).expect("invalid arena key")
    }
}

impl<T> IndexMut<Key> for Arena<T> {
    fn index_mut(&mut self, key: Key) -> &mut Self::Output {
        self.get_mut(key).expect("invalid arena key")
    }
}

/// Iterator over shared references to arena elements.
pub struct Iter<'a, T> {
    slots: std::slice::Iter<'a, Slot<T>>,
    index: usize,
    remaining: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Key, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let slot = self.slots.next()?;
            let index = self.index;
            self.index += 1;
            if !slot.empty() {
                self.remaining -= 1;
                let data = unsafe { slot.container.data.deref() };
                return Some((
                    Key {
                        index,
                        version: slot.version,
                    },
                    data,
                ));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for Iter<'_, T> {}

/// Iterator over mutable references to arena elements.
pub struct IterMut<'a, T> {
    slots: std::slice::IterMut<'a, Slot<T>>,
    index: usize,
    remaining: usize,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Key, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let slot = self.slots.next()?;
            let index = self.index;
            self.index += 1;
            if !slot.empty() {
                self.remaining -= 1;
                let data = unsafe { slot.container.data.deref_mut() };
                return Some((
                    Key {
                        index,
                        version: slot.version,
                    },
                    data,
                ));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for IterMut<'_, T> {}

/// Owning iterator over arena elements.
pub struct IntoIter<T> {
    slots: std::vec::IntoIter<Slot<T>>,
    index: usize,
    remaining: usize,
}

impl<T> Iterator for IntoIter<T> {
    type Item = (Key, T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let mut slot = self.slots.next()?;
            let index = self.index;
            self.index += 1;
            if !slot.empty() {
                self.remaining -= 1;
                let data = unsafe { ManuallyDrop::take(&mut slot.container.data) };
                slot.version += 1; // mark empty so Drop doesn't double-free
                return Some((
                    Key {
                        index,
                        version: slot.version - 1,
                    },
                    data,
                ));
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {}

impl<T> Arena<T> {
    /// Remove all elements from the arena, keeping the allocated memory.
    /// Old keys will be invalid after this operation.
    pub fn clear(&mut self) {
        self.slots.clear();
        self.head = 0;
        self.count = 0;
    }

    /// Returns an iterator over shared references to the arena elements.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            slots: self.slots.iter(),
            index: 0,
            remaining: self.count,
        }
    }

    /// Returns an iterator over mutable references to the arena elements.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut {
            slots: self.slots.iter_mut(),
            index: 0,
            remaining: self.count,
        }
    }

    /// Returns an iterator over shared references to the values in the arena.
    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.iter().map(|(_, v)| v)
    }

    /// Returns an iterator over mutable references to the values in the arena.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.iter_mut().map(|(_, v)| v)
    }

    /// Returns an iterator over the keys in the arena.
    pub fn keys(&self) -> impl Iterator<Item = Key> {
        self.iter().map(|(k, _)| k)
    }

    /// Retains only the elements specified by the predicate.
    pub fn retain(&mut self, mut f: impl FnMut(Key, &mut T) -> bool) {
        for i in 0..self.slots.len() {
            let slot = &mut self.slots[i];
            if slot.empty() {
                continue;
            }
            let key = Key {
                index: i,
                version: slot.version,
            };
            if !f(key, unsafe { &mut slot.container.data }) {
                unsafe { ManuallyDrop::drop(&mut slot.container.data) };
                slot.container = Container { next: self.head };
                slot.version += 1;
                self.head = i;
                self.count -= 1;
            }
        }
    }
}

impl<T> IntoIterator for Arena<T> {
    type Item = (Key, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            slots: self.slots.into_iter(),
            index: 0,
            remaining: self.count,
        }
    }
}

impl<'a, T> IntoIterator for &'a Arena<T> {
    type Item = (Key, &'a T);
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut Arena<T> {
    type Item = (Key, &'a mut T);
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for Arena<T> {
    fn clone(&self) -> Self {
        Self {
            slots: self.slots.clone(),
            head: self.head,
            count: self.count,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.slots.clone_from(&source.slots);
        self.head = source.head;
        self.count = source.count;
    }
}

impl<T: PartialEq> PartialEq for Arena<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.count != other.count {
            return false;
        }
        self.iter().all(|(key, val)| other.get(key) == Some(val))
    }
}

impl<T: Eq> Eq for Arena<T> {}

/// Draining iterator that removes all elements from the arena.
pub struct Drain<'a, T> {
    arena: &'a mut Arena<T>,
    index: usize,
}

impl<T> Iterator for Drain<'_, T> {
    type Item = (Key, T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index >= self.arena.slots.len() {
                return None;
            }
            let i = self.index;
            self.index += 1;
            let slot = &mut self.arena.slots[i];
            if slot.empty() {
                continue;
            }
            let key = Key {
                index: i,
                version: slot.version,
            };
            let value = unsafe { ManuallyDrop::take(&mut slot.container.data) };
            slot.container = Container {
                next: self.arena.head,
            };
            slot.version += 1;
            self.arena.head = i;
            self.arena.count -= 1;
            return Some((key, value));
        }
    }
}

impl<T> Drop for Drain<'_, T> {
    fn drop(&mut self) {
        // Exhaust remaining elements.
        self.for_each(drop);
    }
}

impl<T> Arena<T> {
    /// Drains all elements from the arena, returning them as an iterator.
    /// The arena keeps its allocated memory for reuse.
    pub fn drain(&mut self) -> Drain<'_, T> {
        Drain {
            arena: self,
            index: 0,
        }
    }
}

impl<T> Extend<T> for Arena<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for value in iter {
            self.insert(value);
        }
    }
}

impl<T: Debug> Debug for Arena<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}

impl<T> FromIterator<T> for Arena<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut arena = Self::with_capacity(iter.size_hint().0);
        arena.extend(iter);
        arena
    }
}
