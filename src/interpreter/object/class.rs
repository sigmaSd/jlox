
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::interpreter::{
    lox_callable::{LoxCallable, LoxFunction},
    Object,
};

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

#[derive(Debug, Clone)]
pub struct LoxInstance {
    pub class: LoxClass,
    fields: Arc<RwLock<HashMap<String, Object>>>,
}

impl LoxInstance {
    pub fn new(class: LoxClass) -> Self {
        Self {
            class,
            fields: Default::default(),
        }
    }
    pub fn get(&self, name: &crate::scanner::Token) -> Object {
        if let Some(field) = self.fields.try_read().unwrap().get(&name.lexeme) {
            return field.clone();
        }
        let method = self.class.find_method(&name.lexeme);
        if let Some(method) = method {
            return Object::Function(Arc::new(RwLock::new(method.bind(self.clone()))));
        }
        panic!("{} Undefined property '{}'", name, name.lexeme)
    }

    pub(crate) fn set(&mut self, name: crate::scanner::Token, value: Object) {
        self.fields.try_write().unwrap().insert(name.lexeme, value);
    }
}
impl std::fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", self.class.name)
    }
}
