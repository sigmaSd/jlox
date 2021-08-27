use std::{
    sync::{Arc, RwLock},
    time::SystemTime,
};

use trycatch::{catch, CatchError, Exception};

use super::ReturnException;
use crate::{null_obj, obj, stmt};

use super::{environment::Environment, Interpreter, Object};

pub trait LoxCallable: Send + Sync {
    fn arity(&self) -> usize;
    fn call(&self, _interpreter: &mut Interpreter, _arguemnts: Vec<Object>) -> Object;
}

pub struct LoxFunction {
    declaration: stmt::Function,
    closure: Arc<RwLock<Environment>>,
}

impl LoxFunction {
    pub fn new(declaration: stmt::Function, closure: Arc<RwLock<Environment>>) -> Self {
        Self {
            declaration,
            closure,
        }
    }
}

impl LoxCallable for LoxFunction {
    fn arity(&self) -> usize {
        self.declaration.params.len()
    }

    fn call(&self, interpreter: &mut Interpreter, arguemnts: Vec<Object>) -> Object {
        let mut environment = Environment::new(Some(self.closure.clone()));
        for (param, arg) in self.declaration.params.iter().zip(arguemnts.into_iter()) {
            environment.define(param.lexeme.clone(), Some(arg));
        }

        let body = &self.declaration.body;
        let mut interpreter = interpreter.clone();

        let execution_result =
            catch!(interpreter.execute_block(body, environment) => ReturnException);

        if let Err(e) = execution_result {
            match e {
                CatchError::Exception(e) => e.payload(),
                CatchError::Panic(p) => std::panic::panic_any(p),
            }
        } else {
            null_obj!()
        }
    }
}
impl ToString for LoxFunction {
    fn to_string(&self) -> String {
        format!("<fn {}>", self.declaration.name.lexeme)
    }
}

pub struct Clock {}
impl LoxCallable for Clock {
    fn arity(&self) -> usize {
        0
    }

    fn call(&self, _interpreter: &mut Interpreter, _arguemnts: Vec<Object>) -> Object {
        obj!(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as f64
                / 1000. ; Object::Number
        )
    }
}
