//! Module for analyzing circuits and extracting useful information.
//!
//! This module provides various analyses that can be performed
//! on computation circuits represented as graphs.
//! Each analysis is implemented as a struct that implements
//! the [`Analysis`] trait. The analyses can be run using the
//! `Analyzer` struct, which caches results for efficiency.

pub mod analyses;

#[cfg(test)]
mod tests;

use crate::{
    error::{Error, Result},
    gate::Gate,
    graph::circuit::Circuit,
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

/// Trait for analyses that can be performed on circuits.
pub trait Analysis: 'static {
    /// The output type of the analysis.
    type Output;
    /// Run the analysis on the given circuit using the provided analyzer for recursive dependant analyses.
    fn run<T: Gate>(circuit: &Circuit<T>, analyzer: &mut Analyzer<T>) -> Result<Self::Output>;
}

/// Struct that manages and caches analyses on circuits.
pub struct Analyzer<T: Gate> {
    /// Cache mapping [`TypeId`] of analyses to their results.
    cache: HashMap<TypeId, Box<dyn Any>>,
    /// Phantom data to associate with the gate type.
    _marker: std::marker::PhantomData<T>,
}

impl<T: Gate> Analyzer<T> {
    /// Create a new [`Analyzer`].
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the result of an analysis, computing and caching it if necessary.
    pub fn get<A>(&mut self, circuit: &Circuit<T>) -> Result<&A::Output>
    where
        A: Analysis,
    {
        let key = TypeId::of::<A>();

        if self.cache.contains_key(&key) {
            return self
                .cache
                .get(&key)
                .ok_or(Error::AnalysisCacheMissingEntry(key))?
                .downcast_ref::<A::Output>()
                .ok_or(Error::AnalysisCacheTypeMismatch(key));
        }

        let result = A::run(circuit, self)?;

        self.cache.insert(key, Box::new(result));

        self.cache
            .get(&key)
            .ok_or(Error::AnalysisCacheMissingEntry(key))?
            .downcast_ref::<A::Output>()
            .ok_or(Error::AnalysisCacheTypeMismatch(key))
    }

    /// Invalidate all cached analyses.
    pub fn invalidate_all(&mut self) {
        self.cache.clear();
    }

    /// Invalidate all cached analyses except for the ones with the given [`TypeId`]s.
    pub fn invalidate_except(&mut self, preserved: &[TypeId]) {
        self.cache.retain(|key, _| preserved.contains(key));
    }
}

impl<T: Gate> Default for Analyzer<T> {
    fn default() -> Self {
        Self::new()
    }
}
