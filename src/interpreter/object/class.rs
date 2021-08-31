use std::collections::HashMap;

use crate::interpreter::Object;

use super::{function::LoxFunction, instance::LoxInstance, lox_callable::LoxCallable};

#[derive(Debug, Clone)]
pub struct LoxClass {
    pub name: String,
    methods: HashMap<String, LoxFunction>,
    superclass: Option<Box<LoxClass>>,
}

impl LoxClass {
    pub fn new(
        name: String,
        superclass: Option<LoxClass>,
        methods: HashMap<String, LoxFunction>,
    ) -> Self {
        Self {
            name,
            methods,
            superclass: superclass.map(Box::new),
        }
    }

    pub(crate) fn find_method(&self, name: &str) -> Option<LoxFunction> {
        if let Some(method) = self.methods.get(name) {
            return Some(method.clone());
        }
        if let Some(ref superclass) = self.superclass {
            return superclass.find_method(name);
        }
        None
    }
}
impl std::fmt::Display for LoxClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl LoxCallable for LoxClass {
    fn arity(&self) -> usize {
        if let Some(initializer) = self.find_method("init") {
            initializer.arity()
        } else {
            0
        }
    }

    fn call(
        &self,
        interpreter: &mut crate::interpreter::Interpreter,
        arguemnts: Vec<Object>,
    ) -> Object {
        let instance = LoxInstance::new(self.clone());

        if let Some(initializer) = self.find_method("init") {
            initializer
                .bind(instance.clone())
                .call(interpreter, arguemnts);
        }
        Object::Instance(instance)
    }
}
