use crate::Arena;

#[test]
fn new_default() {
    let arena: Arena<i32> = Arena::new();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
    assert_eq!(arena.capacity(), 0);

    let arena: Arena<i32> = Arena::default();
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
    assert_eq!(arena.capacity(), 0);
}

#[test]
fn with_capacity() {
    let mut arena: Arena<i32> = Arena::with_capacity(10);
    assert_eq!(arena.capacity(), 10);
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);

    for i in 0..10 {
        arena.insert(i);
    }
    assert_eq!(arena.capacity(), 10);
    assert_eq!(arena.len(), 10);

    arena.insert(10);
    assert!(arena.capacity() > 10);
}

#[test]
fn reserve() {
    let mut arena: Arena<i32> = Arena::new();
    assert_eq!(arena.capacity(), 0);

    arena.reserve(10);
    assert!(arena.capacity() >= 10);

    for i in 0..10 {
        arena.insert(i);
    }

    arena.reserve(5);
    assert!(arena.capacity() >= 15);
}

#[test]
fn try_reserve() {
    let mut arena: Arena<i32> = Arena::new();
    assert!(arena.try_reserve(10).is_ok());
    assert!(arena.capacity() >= 10);

    assert!(arena.try_reserve(usize::MAX).is_err());
}

#[test]
fn capacity_len_clean() {
    let mut arena: Arena<i32> = Arena::new();
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());

    let k1 = arena.insert(10);
    assert_eq!(arena.len(), 1);
    assert!(!arena.is_empty());
    assert!(arena.capacity() >= 1);

    let k2 = arena.insert(20);
    assert_eq!(arena.len(), 2);
    assert!(arena.capacity() >= 2);

    arena.remove(k1);
    assert_eq!(arena.len(), 1);
    assert!(!arena.is_empty());

    arena.remove(k2);
    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
    assert!(arena.capacity() >= 2);
}

#[test]
fn insert_basic() {
    let mut arena: Arena<i32> = Arena::new();
    let key = arena.insert(42);

    assert_eq!(key.index(), 0);
    assert_eq!(key.version(), 1);
    assert_eq!(arena.len(), 1);
    assert!(!arena.is_empty());

    assert_eq!(arena.get(key), Some(&42));
    assert_eq!(arena.get_mut(key), Some(&mut 42));
}

#[test]
fn insert_multiple_no_remove() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    let k2 = arena.insert(20);
    let k3 = arena.insert(30);

    assert_eq!(k1.index(), 0);
    assert_eq!(k2.index(), 1);
    assert_eq!(k3.index(), 2);

    assert_eq!(arena.len(), 3);
}

#[test]
fn insert_with_key() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert_with_key(|key| {
        assert_eq!(key.index(), 0);
        assert_eq!(key.version(), 1);
        10
    });
    assert_eq!(k1.index(), 0);
    assert_eq!(arena.get(k1), Some(&10));
}

#[test]
fn insert_reuses_freelist() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    let k2 = arena.insert(20);

    assert_eq!(k1.index(), 0);
    assert_eq!(k2.index(), 1);

    arena.remove(k1);

    let k3 = arena.insert(30);
    assert_eq!(k3.index(), 0);
    assert_eq!(k3.version(), 3);

    assert_eq!(arena.len(), 2);
    assert_eq!(arena.get(k3), Some(&30));
    assert_eq!(arena.get(k2), Some(&20));
}

#[test]
fn insert_exhaust_freelist_then_append() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    let _ = arena.insert(20);

    arena.remove(k1);

    let k3 = arena.insert(30);
    assert_eq!(k3.index(), 0);

    let k4 = arena.insert(40);
    assert_eq!(k4.index(), 2);
    assert_eq!(arena.len(), 3);
}

#[test]
fn get_valid() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    assert_eq!(arena.get(k1), Some(&10));
}

#[test]
fn get_mut_valid() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    if let Some(val) = arena.get_mut(k1) {
        *val += 1;
    }
    assert_eq!(arena.get(k1), Some(&11));
}

#[test]
fn get_unknown_index() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(10);

    let mut arena2: Arena<i32> = Arena::new();
    for _ in 0..100 {
        arena2.insert(0);
    }
    let k_large = arena2.insert(0);

    assert_eq!(arena.get(k_large), None);
}

#[test]
fn get_stale_version() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    arena.remove(k1);
    let k2 = arena.insert(20);

    assert_eq!(arena.get(k1), None);
    assert_eq!(arena.get(k2), Some(&20));
}

#[test]
fn get_deleted_slot() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    arena.remove(k1);
    assert_eq!(arena.get(k1), None);
}

#[test]
fn contains_key_valid() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    assert!(arena.contains_key(k1));
}

#[test]
fn contains_key_stale() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    arena.remove(k1);
    let _ = arena.insert(20);
    assert!(!arena.contains_key(k1));
}

#[test]
fn contains_key_deleted() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    arena.remove(k1);
    assert!(!arena.contains_key(k1));
}

#[test]
fn index_trait() {
    let mut arena: Arena<i32> = Arena::new();
    let k = arena.insert(10);
    assert_eq!(arena[k], 10);
}

#[test]
fn index_mut_trait() {
    let mut arena: Arena<i32> = Arena::new();
    let k = arena.insert(10);
    arena[k] = 20;
    assert_eq!(arena[k], 20);
}

#[test]
#[should_panic]
fn index_panic_invalid() {
    let mut arena: Arena<i32> = Arena::new();
    let k = arena.insert(10);
    arena.remove(k);
    let _ = arena[k];
}

#[test]
fn remove_valid_returns_some() {
    let mut arena: Arena<i32> = Arena::new();
    let k = arena.insert(10);
    assert_eq!(arena.remove(k), Some(10));
    assert!(arena.is_empty());
}

#[test]
fn remove_invalid_returns_none() {
    let mut arena: Arena<i32> = Arena::new();
    let k = arena.insert(10);
    arena.remove(k);
    assert_eq!(arena.remove(k), None);
}

#[test]
fn remove_already_removed() {
    let mut arena: Arena<i32> = Arena::new();
    let k = arena.insert(10);
    assert_eq!(arena.remove(k), Some(10));
    assert_eq!(arena.remove(k), None);
}

#[test]
fn remove_stale_returns_none() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    arena.remove(k1);
    let _k2 = arena.insert(20);

    assert_eq!(arena.remove(k1), None);
    assert_eq!(arena.len(), 1);
}

#[test]
fn remove_updates_head() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    let k2 = arena.insert(20);

    arena.remove(k1);
    let k3 = arena.insert(30);
    assert_eq!(k3.index(), 0);

    arena.remove(k2);
    let k4 = arena.insert(40);
    assert_eq!(k4.index(), 1);
}

#[test]
fn iter_empty() {
    let arena: Arena<i32> = Arena::new();
    assert_eq!(arena.iter().count(), 0);
}

#[test]
fn iter_filled() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(10);
    arena.insert(20);
    arena.insert(30);

    let mut items: Vec<&i32> = arena.iter().map(|(_, v)| v).collect();
    items.sort();
    assert_eq!(items, vec![&10, &20, &30]);
}

#[test]
fn iter_skip_empty() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    let _ = arena.insert(20);
    arena.remove(k1);

    let items: Vec<&i32> = arena.iter().map(|(_, v)| v).collect();
    assert_eq!(items, vec![&20]);
}

#[test]
fn iter_mut_modification() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(10);
    arena.insert(20);

    for (_, v) in arena.iter_mut() {
        *v += 1;
    }

    let mut items: Vec<&i32> = arena.iter().map(|(_, v)| v).collect();
    items.sort();
    assert_eq!(items, vec![&11, &21]);
}

#[test]
fn into_iter_consumes() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(10);
    arena.insert(20);

    let mut items: Vec<i32> = arena.into_iter().map(|(_, v)| v).collect();
    items.sort();
    assert_eq!(items, vec![10, 20]);
}

#[test]
fn into_iter_drops_remaining() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct DropTracker(Arc<AtomicUsize>);
    impl Drop for DropTracker {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    let drops = Arc::new(AtomicUsize::new(0));
    let mut arena = Arena::new();
    arena.insert(DropTracker(drops.clone()));
    arena.insert(DropTracker(drops.clone()));

    let mut iter = arena.into_iter();
    assert!(iter.next().is_some());
    drop(iter);

    assert_eq!(drops.load(Ordering::SeqCst), 2);
}

#[test]
fn keys_iterator() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    let k2 = arena.insert(20);

    let keys: Vec<crate::Key> = arena.keys().collect();
    assert!(keys.contains(&k1));
    assert!(keys.contains(&k2));
    assert_eq!(keys.len(), 2);
}

#[test]
fn values_iterator() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(10);
    arena.insert(20);

    let mut values: Vec<&i32> = arena.values().collect();
    values.sort();
    assert_eq!(values, vec![&10, &20]);
}

#[test]
fn values_mut_iterator() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(10);
    arena.insert(20);

    for v in arena.values_mut() {
        *v *= 2;
    }

    let mut values: Vec<&i32> = arena.values().collect();
    values.sort();
    assert_eq!(values, vec![&20, &40]);
}

#[test]
fn drain_all() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(10);
    arena.insert(20);
    arena.insert(30);

    let drained: Vec<i32> = arena.drain().map(|(_, v)| v).collect();
    assert_eq!(drained.len(), 3);
    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn drain_partial() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(10);
    arena.insert(20);
    arena.insert(30);

    {
        let mut drain = arena.drain();
        assert!(drain.next().is_some());
    }

    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
}

#[test]
fn retain_all() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    let k2 = arena.insert(20);

    arena.retain(|_, _| true);

    assert_eq!(arena.len(), 2);
    assert_eq!(arena.get(k1), Some(&10));
    assert_eq!(arena.get(k2), Some(&20));
}

#[test]
fn retain_none() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(10);
    arena.insert(20);

    arena.retain(|_, _| false);

    assert_eq!(arena.len(), 0);
    assert!(arena.is_empty());
}

#[test]
fn retain_conditional() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    let k2 = arena.insert(20);
    let k3 = arena.insert(30);

    arena.retain(|_, v| *v % 20 != 0); // Remove 20

    assert_eq!(arena.len(), 2);
    assert_eq!(arena.get(k1), Some(&10));
    assert_eq!(arena.get(k2), None);
    assert_eq!(arena.get(k3), Some(&30));
}

#[test]
fn clear_removes_all() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(10);
    arena.insert(20);

    arena.clear();

    assert!(arena.is_empty());
    assert_eq!(arena.len(), 0);
    assert_eq!(arena.iter().count(), 0);
}

#[test]
fn clear_retains_capacity() {
    let mut arena: Arena<i32> = Arena::new();
    arena.reserve(100);
    let cap = arena.capacity();

    arena.insert(10);
    arena.clear();

    assert_eq!(arena.capacity(), cap);
}

#[test]
fn clear_generation_reset_behavior() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    let index = k1.index();
    let version = k1.version();

    arena.clear();

    let k2 = arena.insert(20);

    assert_eq!(k2.index(), index);
    assert_eq!(k2.version(), version);
    assert_eq!(k1, k2);
}

#[test]
fn drop_arena_drops_values() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct DropTracker(Arc<AtomicUsize>);
    impl Drop for DropTracker {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    let drops = Arc::new(AtomicUsize::new(0));

    {
        let mut arena = Arena::new();
        arena.insert(DropTracker(drops.clone()));
        arena.insert(DropTracker(drops.clone()));
    }

    assert_eq!(drops.load(Ordering::SeqCst), 2);
}

#[test]
fn drop_iter_drops_elements() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct DropTracker(Arc<AtomicUsize>);
    impl Drop for DropTracker {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    let drops = Arc::new(AtomicUsize::new(0));
    let mut arena = Arena::new();
    arena.insert(DropTracker(drops.clone()));
    arena.insert(DropTracker(drops.clone()));

    let iter = arena.into_iter();
    drop(iter);

    assert_eq!(drops.load(Ordering::SeqCst), 2);
}

#[test]
fn drop_drain_drops_elements() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct DropTracker(Arc<AtomicUsize>);
    impl Drop for DropTracker {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    let drops = Arc::new(AtomicUsize::new(0));
    let mut arena = Arena::new();
    arena.insert(DropTracker(drops.clone()));
    arena.insert(DropTracker(drops.clone()));

    {
        let drain = arena.drain();
        drop(drain);
    }

    assert_eq!(drops.load(Ordering::SeqCst), 2);
    assert!(arena.is_empty());
}

#[test]
fn clone_integrity() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    let k2 = arena.insert(20);

    let cloned = arena.clone();

    assert_eq!(cloned.len(), 2);
    assert_eq!(cloned.get(k1), Some(&10));
    assert_eq!(cloned.get(k2), Some(&20));
}

#[test]
fn clone_independence() {
    let mut arena: Arena<i32> = Arena::new();
    let k1 = arena.insert(10);
    let mut cloned = arena.clone();

    if let Some(val) = cloned.get_mut(k1) {
        *val = 30;
    }

    assert_eq!(arena.get(k1), Some(&10));
    assert_eq!(cloned.get(k1), Some(&30));

    if let Some(val) = arena.get_mut(k1) {
        *val = 40;
    }
    assert_eq!(arena.get(k1), Some(&40));
    assert_eq!(cloned.get(k1), Some(&30));
}

#[test]
fn clone_from_reuses_capacity() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(1);

    let mut cloner = Arena::new();
    cloner.reserve(100);
    let initial_cap = cloner.capacity();

    cloner.clone_from(&arena);

    assert_eq!(cloner.len(), 1);
    assert!(cloner.capacity() >= initial_cap);
}

#[test]
fn eq_reflexive() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(10);
    assert_eq!(arena, arena);
}

#[test]
fn eq_different_values() {
    let mut arena1: Arena<i32> = Arena::new();
    let _ = arena1.insert(10);

    let mut arena2: Arena<i32> = Arena::new();
    let _ = arena2.insert(20);

    assert_ne!(arena1, arena2);
}

#[test]
fn eq_different_keys() {
    let mut arena1: Arena<i32> = Arena::new();
    let _ = arena1.insert(10);

    let mut arena2: Arena<i32> = Arena::new();
    let k2 = arena2.insert(10);
    arena2.remove(k2);
    let _ = arena2.insert(10);

    assert_ne!(arena1, arena2);
}

#[test]
fn zst_behavior() {
    let mut arena: Arena<()> = Arena::new();
    let k1 = arena.insert(());
    let k2 = arena.insert(());

    assert_eq!(arena.len(), 2);
    assert_ne!(k1, k2);
    assert_eq!(arena.get(k1), Some(&()));
    arena.remove(k1);
    assert_eq!(arena.get(k1), None);
    assert_eq!(arena.len(), 1);
}

#[test]
fn large_number_of_items() {
    let mut arena: Arena<i32> = Arena::new();
    let count = 1000;
    let mut keys = Vec::with_capacity(count);

    for i in 0..count {
        keys.push(arena.insert(i as i32));
    }

    assert_eq!(arena.len(), count);

    for (i, &key) in keys.iter().enumerate() {
        assert_eq!(arena.get(key), Some(&(i as i32)));
    }

    // Remove half
    for i in (0..count).step_by(2) {
        arena.remove(keys[i]);
    }

    assert_eq!(arena.len(), count / 2);
}

#[test]
fn key_debug() {
    let mut arena: Arena<i32> = Arena::new();
    let k = arena.insert(10);
    let debug_str = format!("{:?}", k);
    assert!(!debug_str.is_empty());
}

#[test]
fn arena_debug() {
    let mut arena: Arena<i32> = Arena::new();
    arena.insert(10);
    let debug_str = format!("{:?}", arena);
    assert!(!debug_str.is_empty());
}

#[test]
fn box_clone_and_drop() {
    let mut arena: Arena<Box<i32>> = Arena::new();
    arena.insert(Box::new(10));
    let cloned = arena.clone();
    assert_eq!(arena.len(), 1);
    assert_eq!(cloned.len(), 1);
}
