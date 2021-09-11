use std::{
    fmt,
    sync::{Arc, RwLock},
    time::SystemTime,
};

use trycatch::{catch, throw, CatchError, ExceptionDowncast};

use crate::{
    ar,
    interpreter::{environment::Environment, Interpreter, ReturnException, RuntimeError},
    null_obj, obj, stmt,
};

use super::{instance::LoxInstance, lox_callable::LoxCallable, Object, ObjectInner};

#[derive(Debug, Clone)]
pub struct LoxFunction {
    declaration: stmt::Function,
    closure: Arc<RwLock<Environment>>,
    is_initializer: bool,
}
impl fmt::Display for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<fn {}>", self.declaration.name.lexeme)
    }
}

impl LoxFunction {
    pub fn new(
        declaration: stmt::Function,
        closure: Arc<RwLock<Environment>>,
        is_initializer: bool,
    ) -> Self {
        Self {
            declaration,
            closure,
            is_initializer,
        }
    }
    pub fn bind(&self, instance: LoxInstance) -> LoxFunction {
        let mut environment = Environment::new(Some(self.closure.clone()));
        environment.define("this".into(), Some(ar!(ObjectInner::Instance(instance))));
        LoxFunction {
            declaration: self.declaration.clone(),
            closure: Arc::new(RwLock::new(environment)),
            is_initializer: self.is_initializer,
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

        let execution_result = catch(move || interpreter.execute_block(body, environment));

        if let Err(e) = execution_result {
            return match e {
                CatchError::Exception(e) => {
                    if self.is_initializer {
                        self.closure.try_read().unwrap().get_at(&0, "this")
                    } else {
                        // Exception can be either ReturnException or RuntimeError
                        match e.try_downcast::<ReturnException>() {
                            Ok(ret) => ret.0,
                            Err(exception) => throw(*exception.downcast::<RuntimeError>().unwrap()),
                        }
                    }
                }
                CatchError::Panic(p) => std::panic::panic_any(p),
            };
        }
        if self.is_initializer {
            return self.closure.try_read().unwrap().get_at(&0, "this");
        }
        null_obj!()
    }
}

pub struct Clock {}
impl fmt::Display for Clock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<native fn>")
    }
}
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
                / 1000. ; ObjectInner::Number
        )
    }
}
