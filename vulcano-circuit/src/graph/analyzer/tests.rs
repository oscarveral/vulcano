use std::any::TypeId;

use crate::{
    error::Result,
    gate::Gate,
    graph::{
        analyzer::{Analysis, Analyzer},
        builder::Builder,
        circuit::Circuit,
    },
};

struct DummyGate;

impl Gate for DummyGate {
    fn arity(&self) -> usize {
        1
    }

    fn name(&self) -> &str {
        "DummyGate"
    }
}

struct TestAnalysisUsize;
struct TestAnalysisString;

impl Analysis for TestAnalysisUsize {
    type Output = usize;

    fn run<T: Gate>(_circuit: &Circuit<T>, _analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        Ok(42)
    }
}

impl Analysis for TestAnalysisString {
    type Output = String;

    fn run<T: Gate>(_circuit: &Circuit<T>, _analyzer: &mut Analyzer<T>) -> Result<Self::Output> {
        Ok("Hello, World!".to_string())
    }
}

#[test]
fn analyzer_creation() {
    let analyzer = Analyzer::<DummyGate>::new();
    assert!(analyzer.cache.is_empty());
}

#[test]
fn get_analysis() {
    let mut builder = Builder::<DummyGate>::new();

    let input = builder.add_input();
    let gate = builder.add_gate(DummyGate);
    let output = builder.add_output();

    let res = builder.connect_input_to_gate(input, gate);
    assert!(res.is_ok());
    let res = builder.connect_gate_to_output(gate, output);
    assert!(res.is_ok());

    let circuit = builder.finalize();

    assert!(circuit.is_ok());

    let circuit = circuit.unwrap();

    let mut analyzer = Analyzer::<DummyGate>::new();

    let result_usize = analyzer.get::<TestAnalysisUsize>(&circuit);

    assert!(result_usize.is_ok());
    let result_usize = result_usize.unwrap();
    assert_eq!(*result_usize, 42);

    let result_string = analyzer.get::<TestAnalysisString>(&circuit);

    assert!(result_string.is_ok());
    let result_string = result_string.unwrap();
    assert_eq!(result_string.as_str(), "Hello, World!");

    assert_eq!(analyzer.cache.len(), 2);
}

#[test]
fn analyzer_cache() {
    let mut builder = Builder::<DummyGate>::new();

    let input = builder.add_input();
    let gate = builder.add_gate(DummyGate);
    let output = builder.add_output();

    let res = builder.connect_input_to_gate(input, gate);
    assert!(res.is_ok());
    let res = builder.connect_gate_to_output(gate, output);
    assert!(res.is_ok());

    let circuit = builder.finalize();

    assert!(circuit.is_ok());

    let circuit = circuit.unwrap();

    let mut analyzer = Analyzer::<DummyGate>::new();

    let result_usize = analyzer.get::<TestAnalysisUsize>(&circuit);

    assert!(result_usize.is_ok());
    let result_usize = result_usize.unwrap();
    assert_eq!(*result_usize, 42);

    let result_usize_cached = analyzer.get::<TestAnalysisUsize>(&circuit);
    assert!(result_usize_cached.is_ok());
    let result_usize_cached = result_usize_cached.unwrap();
    assert_eq!(*result_usize_cached, 42);

    // Ensure that the results are cached
    assert_eq!(analyzer.cache.len(), 1);
}

#[test]
fn invalidate_all_analyses() {
    let mut builder = Builder::<DummyGate>::new();

    let input = builder.add_input();
    let gate = builder.add_gate(DummyGate);
    let output = builder.add_output();

    let res = builder.connect_input_to_gate(input, gate);
    assert!(res.is_ok());
    let res = builder.connect_gate_to_output(gate, output);
    assert!(res.is_ok());

    let circuit = builder.finalize();

    assert!(circuit.is_ok());

    let circuit = circuit.unwrap();

    let mut analyzer = Analyzer::<DummyGate>::new();

    let result_usize = analyzer.get::<TestAnalysisUsize>(&circuit);

    assert!(result_usize.is_ok());
    let result_usize = result_usize.unwrap();
    assert_eq!(*result_usize, 42);

    let result_string = analyzer.get::<TestAnalysisString>(&circuit);

    assert!(result_string.is_ok());
    let result_string = result_string.unwrap();
    assert_eq!(result_string.as_str(), "Hello, World!");

    assert_eq!(analyzer.cache.len(), 2);

    analyzer.invalidate_all();

    assert!(analyzer.cache.is_empty());
}

#[test]
fn invalidate_except() {
    let mut builder = Builder::<DummyGate>::new();

    let input = builder.add_input();
    let gate = builder.add_gate(DummyGate);
    let output = builder.add_output();

    let res = builder.connect_input_to_gate(input, gate);
    assert!(res.is_ok());
    let res = builder.connect_gate_to_output(gate, output);
    assert!(res.is_ok());

    let circuit = builder.finalize();

    assert!(circuit.is_ok());

    let circuit = circuit.unwrap();

    let mut analyzer = Analyzer::<DummyGate>::new();

    let result_usize = analyzer.get::<TestAnalysisUsize>(&circuit);

    assert!(result_usize.is_ok());
    let result_usize = result_usize.unwrap();
    assert_eq!(*result_usize, 42);

    let result_string = analyzer.get::<TestAnalysisString>(&circuit);

    assert!(result_string.is_ok());
    let result_string = result_string.unwrap();
    assert_eq!(result_string.as_str(), "Hello, World!");

    assert_eq!(analyzer.cache.len(), 2);

    analyzer.invalidate_except(&[TypeId::of::<TestAnalysisUsize>()]);

    assert_eq!(analyzer.cache.len(), 1);
    assert!(
        analyzer
            .cache
            .contains_key(&TypeId::of::<TestAnalysisUsize>())
    );
}
