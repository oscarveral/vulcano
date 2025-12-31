//! Tests for the vulcano-arena crate.

use std::collections::HashMap;

use crate::Arena;

#[test]
fn insert_and_get() {
    let mut arena = Arena::new();
    let key = arena.insert(42);
    assert_eq!(arena.get(key), Some(&42));
    assert_eq!(arena.len(), 1);
}

#[test]
fn remove_and_reuse() {
    let mut arena = Arena::new();
    let key1 = arena.insert("first");
    arena.remove(key1);

    // Slot is reused.
    let key2 = arena.insert("second");
    assert_eq!(key2.index(), key1.index());

    // Old key is stale.
    assert_eq!(arena.get(key1), None);
    assert_eq!(arena.get(key2), Some(&"second"));
}

#[test]
fn generation_prevents_aba() {
    let mut arena = Arena::new();

    let key_a = arena.insert("A");
    arena.remove(key_a);
    let key_b = arena.insert("B");

    // Same index, different generation.
    assert_eq!(key_a.index(), key_b.index());
    assert_ne!(key_a.generation(), key_b.generation());

    // Old key does not access new value.
    assert_eq!(arena.get(key_a), None);
    assert_eq!(arena.get(key_b), Some(&"B"));
}

#[test]
fn iter_only_occupied() {
    let mut arena = Arena::new();
    let k1 = arena.insert(1);
    let k2 = arena.insert(2);
    let k3 = arena.insert(3);

    arena.remove(k2);

    let keys: Vec<_> = arena.iter().map(|(k, _)| k).collect();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&k1));
    assert!(keys.contains(&k3));
}

#[test]
fn get_mut_modifies_value() {
    let mut arena = Arena::new();
    let key = arena.insert(10);

    if let Some(val) = arena.get_mut(key) {
        *val = 20;
    }

    assert_eq!(arena.get(key), Some(&20));
}

#[test]
fn contains_checks_validity() {
    let mut arena = Arena::new();
    let key = arena.insert("hello");

    assert!(arena.contains(key));
    arena.remove(key);
    assert!(!arena.contains(key));
}

#[test]
fn clear_removes_all() {
    let mut arena = Arena::new();
    let k1 = arena.insert(1);
    let k2 = arena.insert(2);

    arena.clear();

    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
    assert!(!arena.contains(k1));
    assert!(!arena.contains(k2));
}

#[test]
fn with_capacity_preallocates() {
    let arena: Arena<i32> = Arena::with_capacity(100);
    assert!(arena.capacity() >= 100);
    assert_eq!(arena.len(), 0);
}

#[test]
fn iter_mut_modifies_all() {
    let mut arena = Arena::new();
    arena.insert(1);
    arena.insert(2);
    arena.insert(3);

    for (_, val) in arena.iter_mut() {
        *val *= 10;
    }

    let values: Vec<_> = arena.iter().map(|(_, v)| *v).collect();
    assert!(values.contains(&10));
    assert!(values.contains(&20));
    assert!(values.contains(&30));
}

#[test]
fn remove_stale_key_returns_none() {
    let mut arena = Arena::new();
    let key = arena.insert("value");
    arena.remove(key);

    // Stale key on remove returns None.
    assert_eq!(arena.remove(key), None);
}

#[test]
fn double_remove_same_key() {
    let mut arena = Arena::new();
    let key = arena.insert(123);

    assert_eq!(arena.remove(key), Some(123));
    assert_eq!(arena.remove(key), None); // Second remove fails.
}

#[test]
fn get_mut_stale_key_returns_none() {
    let mut arena = Arena::new();
    let key = arena.insert("data");
    arena.remove(key);

    assert_eq!(arena.get_mut(key), None);
}

#[test]
fn multiple_generation_cycles() {
    let mut arena = Arena::new();

    // Cycle through many generations on the same slot.
    let mut last_gen = 0;
    for i in 0..10 {
        let key = arena.insert(i);
        assert_eq!(key.index(), 0); // Always slot 0.
        assert_eq!(key.generation(), last_gen);
        arena.remove(key);
        last_gen = key.generation() + 1;
    }
}

#[test]
fn iter_on_empty() {
    let arena: Arena<i32> = Arena::new();
    assert_eq!(arena.iter().count(), 0);
}

#[test]
fn is_empty_on_new() {
    let arena: Arena<String> = Arena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn default_creates_empty() {
    let arena: Arena<u8> = Arena::default();
    assert!(arena.is_empty());
}

#[test]
fn key_equality() {
    let mut arena = Arena::new();
    let key1 = arena.insert(1);
    let key2 = arena.insert(2);

    assert_eq!(key1, key1);
    assert_ne!(key1, key2);
}

#[test]
fn key_hash_for_hashmap() {
    let mut arena = Arena::new();
    let key1 = arena.insert("one");
    let key2 = arena.insert("two");

    let mut map = HashMap::new();
    map.insert(key1, 1);
    map.insert(key2, 2);

    assert_eq!(map.get(&key1), Some(&1));
    assert_eq!(map.get(&key2), Some(&2));
}

#[test]
fn key_copy_and_clone() {
    let mut arena = Arena::new();
    let key = arena.insert(42);

    let key_copy_1 = key;
    let key_copy_2 = key;

    assert_eq!(arena.get(key_copy_1), Some(&42));
    assert_eq!(arena.get(key_copy_2), Some(&42));
}

#[test]
fn insert_remove_many() {
    let mut arena = Arena::new();
    let mut keys = Vec::new();

    // Insert 1000 items.
    for i in 0..1000 {
        keys.push(arena.insert(i));
    }
    assert_eq!(arena.len(), 1000);

    // Remove every other item.
    for i in (0..1000).step_by(2) {
        arena.remove(keys[i]);
    }
    assert_eq!(arena.len(), 500);

    // Reinsert 500 items (reuses slots).
    for i in 0..500 {
        keys.push(arena.insert(i + 1000));
    }
    assert_eq!(arena.len(), 1000);

    // Verify removed keys are stale.
    for i in (0..1000).step_by(2) {
        assert!(!arena.contains(keys[i]));
    }

    // Verify remaining keys are valid.
    for i in (1..1000).step_by(2) {
        assert!(arena.contains(keys[i]));
    }
}

#[test]
fn reserve_and_fill() {
    let mut arena = Arena::new();
    let key = arena.reserve();

    // Reserved slot should be inaccessible via get/get_mut.
    assert_eq!(arena.get(key), None);
    assert!(arena.get_mut(key).is_none());
    assert!(!arena.contains(key));

    // Fill the slot.
    assert_eq!(arena.fill(key, 42), Ok(()));

    // Slot should now be accessible.
    assert_eq!(arena.get(key), Some(&42));
    assert!(arena.contains(key));
}

#[test]
fn fill_invalid_key_fails() {
    let mut arena = Arena::new();
    let r_key = arena.reserve();
    let i_key = arena.insert(10);

    // Fill with wrong generation (non-existent).
    let bad_key = crate::key::Key {
        index: r_key.index() as u32,
        generation: r_key.generation() + 1,
    };
    assert_eq!(arena.fill(bad_key, 99), Err(99));

    // Fill already occupied slot.
    assert_eq!(arena.fill(i_key, 88), Err(88));

    // Correct fill works.
    assert_eq!(arena.fill(r_key, 100), Ok(()));
}

#[test]
fn remove_reserved_slot() {
    let mut arena: Arena<i32> = Arena::new();
    let key = arena.reserve();

    // Removing a reserved slot should return None but free the slot.
    assert_eq!(arena.remove(key), None);

    // Slot is now free, should be reused.
    let key2 = arena.insert(123);
    assert_eq!(key.index(), key2.index());
    assert_ne!(key.generation(), key2.generation());
}

#[test]
fn iter_skips_reserved() {
    let mut arena = Arena::new();
    let k1 = arena.insert(1);
    let _k2 = arena.reserve(); // reserved
    let k3 = arena.insert(3);

    let keys: Vec<_> = arena.iter().map(|(k, _)| k).collect();
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&k1));
    assert!(keys.contains(&k3));

    let mut count = 0;
    for _ in arena.iter_mut() {
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn double_reservation_reuse() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.reserve();
    let k2 = arena.reserve();

    assert_ne!(k1, k2);

    // Remove one, verify reuse
    arena.remove(k1);
    let k3 = arena.reserve();
    assert_eq!(k1.index(), k3.index());
}
