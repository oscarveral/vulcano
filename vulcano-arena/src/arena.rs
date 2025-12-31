//! Generational arena implementation.

use std::marker::PhantomData;

use crate::key::Key;

/// Internal slot: either occupied with a value or free.
enum Slot<T> {
    /// Slot contains a value.
    Occupied { value: T, generation: u32 },
    /// Slot is free and points to next free slot.
    Free {
        next_free: Option<u32>,
        generation: u32,
    },
}

/// A generational arena for storing values with stable keys.
///
/// The arena provides O(1) insertion, deletion, and lookup. Keys remain
/// valid across insertions and deletions of other elements. Accessing
/// a deleted slot with an old key returns `None` due to generation mismatch.
pub struct Arena<T> {
    /// Storage for all slots.
    slots: Vec<Slot<T>>,
    /// Head of the free list (index of first free slot).
    free_head: Option<u32>,
    /// Number of occupied slots.
    len: usize,
}

impl<T> Arena<T> {
    /// Create a new empty arena.
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            free_head: None,
            len: 0,
        }
    }

    /// Create a new arena with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            free_head: None,
            len: 0,
        }
    }

    /// Insert a value into the arena, returning its key.
    pub fn insert(&mut self, value: T) -> Key<T> {
        self.len += 1;

        if let Some(index) = self.free_head {
            // Reuse a free slot.
            let slot = &mut self.slots[index as usize];
            let generation = match slot {
                Slot::Free {
                    next_free,
                    generation,
                } => {
                    self.free_head = *next_free;
                    *generation
                }
                Slot::Occupied { .. } => unreachable!("free_head pointed to occupied slot"),
            };
            *slot = Slot::Occupied { value, generation };
            Key {
                index,
                generation,
                _marker: PhantomData,
            }
        } else {
            // Grow the arena.
            let index = self.slots.len() as u32;
            self.slots.push(Slot::Occupied {
                value,
                generation: 0,
            });
            Key {
                index,
                generation: 0,
                _marker: PhantomData,
            }
        }
    }

    /// Remove the value associated with the key, returning it if valid.
    pub fn remove(&mut self, key: Key<T>) -> Option<T> {
        let slot = self.slots.get_mut(key.index as usize)?;

        match slot {
            Slot::Occupied { generation, .. } if *generation == key.generation => {
                let new_generation = generation.wrapping_add(1);
                let old = std::mem::replace(
                    slot,
                    Slot::Free {
                        next_free: self.free_head,
                        generation: new_generation,
                    },
                );
                self.free_head = Some(key.index);
                self.len -= 1;

                match old {
                    Slot::Occupied { value, .. } => Some(value),
                    Slot::Free { .. } => unreachable!(),
                }
            }
            _ => None, // Stale key or already free.
        }
    }

    /// Get a reference to the value associated with the key.
    pub fn get(&self, key: Key<T>) -> Option<&T> {
        match self.slots.get(key.index as usize)? {
            Slot::Occupied { value, generation } if *generation == key.generation => Some(value),
            _ => None,
        }
    }

    /// Get a mutable reference to the value associated with the key.
    pub fn get_mut(&mut self, key: Key<T>) -> Option<&mut T> {
        match self.slots.get_mut(key.index as usize)? {
            Slot::Occupied { value, generation } if *generation == key.generation => Some(value),
            _ => None,
        }
    }

    /// Check if a key is valid (points to an occupied slot with matching generation).
    pub fn contains(&self, key: Key<T>) -> bool {
        self.get(key).is_some()
    }

    /// Returns the number of occupied slots.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the arena is empty.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the total capacity (allocated slot space).
    pub fn capacity(&self) -> usize {
        self.slots.capacity()
    }

    /// Clear all elements from the arena.
    pub fn clear(&mut self) {
        self.slots.clear();
        self.free_head = None;
        self.len = 0;
    }

    /// Iterate over all occupied slots, yielding (Key, &T).
    pub fn iter(&self) -> impl Iterator<Item = (Key<T>, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(i, slot)| match slot {
                Slot::Occupied { value, generation } => Some((
                    Key {
                        index: i as u32,
                        generation: *generation,
                        _marker: PhantomData,
                    },
                    value,
                )),
                Slot::Free { .. } => None,
            })
    }

    /// Iterate mutably over all occupied slots, yielding (Key, &mut T).
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Key<T>, &mut T)> {
        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(|(i, slot)| match slot {
                Slot::Occupied { value, generation } => Some((
                    Key {
                        index: i as u32,
                        generation: *generation,
                        _marker: PhantomData,
                    },
                    value,
                )),
                Slot::Free { .. } => None,
            })
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self::new()
    }
}
