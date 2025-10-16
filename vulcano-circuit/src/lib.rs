type GateHandle = usize;
type InputHandle = usize;
type OutputHandle = usize;

pub trait Gate {
    fn arity(&self) -> usize;
}

enum Source {
    Input(InputHandle),
    Gate(GateHandle),
}

enum Destination {
    Output(OutputHandle),
    Gate(GateHandle),
}

type BackwardEdge = Vec<Source>;
type ForwardEdge = Vec<Destination>;

pub enum CircuitError {
    InvalidArity,
    NonExistentGate,
    NonExistentInput,
    NonExistentOutput,
    TooManyConnections,
    DuplicatedOutput,
}

pub struct Circuit<T: Gate> {
    gates: Vec<T>,
    backward_edges: Vec<BackwardEdge>,
    forward_edges: Vec<ForwardEdge>,
    input_count: usize,
    output_count: usize,
}

impl<T: Gate> Circuit<T> {
    pub fn new() -> Self {
        Self {
            gates: Vec::new(),
            backward_edges: Vec::new(),
            forward_edges: Vec::new(),
            input_count: 0,
            output_count: 0,
        }
    }
}

impl<T: Gate> Default for Circuit<T> {
    fn default() -> Self {
        Self::new()
    }
}
