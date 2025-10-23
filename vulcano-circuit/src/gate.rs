pub trait Gate {
    fn arity(&self) -> usize;
    fn name(&self) -> &str;
}
