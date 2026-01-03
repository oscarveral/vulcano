//! Analysis framework
//!
//! This module provides a framework for running analyses on circuits.
//! Analyses are computed on-demand and cached for efficiency.

pub mod analyses;

use crate::{
    circuit::Circuit,
    error::{Error, Result},
    gate::Gate,
};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    rc::Rc,
};

/// Trait for analyses that can be performed on circuits.
pub trait Analysis: 'static {
    /// The output type of the analysis.
    type Output;

    /// Run the analysis on the given circuit.
    fn run<T: Gate>(circuit: &Circuit<T>, analyzer: &mut Analyzer<T>) -> Result<Self::Output>;
}

/// Manages and caches analyses on circuits.
pub struct Analyzer<T: Gate> {
    /// Cache mapping TypeId of analyses to their results.
    cache: HashMap<TypeId, Rc<dyn Any>>,
    /// Phantom data for the gate type.
    _marker: std::marker::PhantomData<T>,
}

impl<T: Gate> Analyzer<T> {
    /// Create a new analyzer.
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the result of an analysis, computing and caching it if necessary.
    pub fn get<A>(&mut self, circuit: &Circuit<T>) -> Result<Rc<A::Output>>
    where
        A: Analysis,
    {
        let key = TypeId::of::<A>();

        if let Some(cached) = self.cache.get(&key) {
            return cached
                .clone()
                .downcast::<A::Output>()
                .map_err(|_| Error::AnalysisCacheTypeMismatch(key));
        }

        let result = A::run(circuit, self)?;
        let rc = Rc::new(result);
        self.cache.insert(key, rc.clone());
        Ok(rc)
    }

    /// Invalidate all cached analyses.
    pub fn invalidate_all(&mut self) {
        self.cache.clear();
    }

    /// Invalidate all cached analyses except for the ones with the given TypeIds.
    pub fn invalidate_except(&mut self, preserved: &[TypeId]) {
        self.cache.retain(|key, _| preserved.contains(key));
    }
}

impl<T: Gate> Default for Analyzer<T> {
    fn default() -> Self {
        Self::new()
    }
}
