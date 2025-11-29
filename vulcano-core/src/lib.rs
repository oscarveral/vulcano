use vulcano_circuit::{gate::Gate, graph::circuit::Circuit};

/// A gate in the Vulcano FHE framework.
///
/// This enum allows the circuit to contain both high-level scheme operations
/// (which may involve metadata updates or complex logic) and low-level backend
/// operations (which are pure math).
pub enum VulcanoGate<S, B> {
    /// A high-level scheme operation.
    Scheme(S),
    /// A low-level backend operation.
    Backend(B),
}

impl<S, B> Gate for VulcanoGate<S, B>
where
    S: Gate,
    B: Gate,
{
    fn arity(&self) -> usize {
        match self {
            VulcanoGate::Scheme(op) => op.arity(),
            VulcanoGate::Backend(op) => op.arity(),
        }
    }

    fn name(&self) -> &str {
        match self {
            VulcanoGate::Scheme(op) => op.name(),
            VulcanoGate::Backend(op) => op.name(),
        }
    }
}

/// A mathematical backend for FHE operations.
///
/// This trait defines the interface that all backends (CPU, GPU, etc.) must implement.
/// It is generic over the specific operations it supports.
pub trait Backend {
    /// The type of operation supported by this backend.
    ///
    /// This enum should implement [`Gate`] and represent the atomic math operations
    /// (e.g., Add, Mul, NTT) or fused kernels available on this backend.
    type BackendOperation: Gate;

    /// The type representing a value in this backend.
    type Value;
}

/// An FHE scheme.
///
/// This trait defines the interface for FHE schemes (e.g., BGV, CKKS, TFHE).
/// It is tied to a specific backend.
pub trait Scheme<B: Backend> {
    /// The type of high-level operation supported by this scheme.
    ///
    /// This enum should implement [`Gate`] and represent scheme-specific logic
    /// (e.g., Rescale, ModSwitch, Bootstrap).
    type SchemeOperation: Gate;

    /// The backend instance used by this scheme.
    fn backend(&self) -> &B;

    // The current circuit being built.
    fn circuit(&self) -> &Circuit<VulcanoGate<Self::SchemeOperation, B::BackendOperation>>;
}
