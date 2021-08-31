use crate::interpreter::Interpreter;

use super::Object;

pub trait LoxCallable: Send + Sync {
    fn arity(&self) -> usize;
    fn call(&self, _interpreter: &mut Interpreter, _arguemnts: Vec<Object>) -> Object;
}
