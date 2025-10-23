#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Node(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Input(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Output(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Wire(pub(crate) usize);
