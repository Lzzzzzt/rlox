use super::{error::Result, interpreter::Interpreter, types::Literal};

pub trait Callable {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Literal>) -> Result<Literal>;
    fn parameter_num(&self) -> usize;
}
