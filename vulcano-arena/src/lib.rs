//! Vulcano Arena - A generational arena (slotmap) implementation.
//!
//! This crate provides a data structure for stable, reusable keys with O(1)
//! insertion, deletion, and lookup. Keys are generational, meaning stale
//! references to deleted slots are detected automatically.

mod arena;
mod key;

#[cfg(test)]
mod tests;

pub use arena::Arena;
pub use key::Key;
