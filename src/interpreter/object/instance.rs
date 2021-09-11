use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use trycatch::throw;

use crate::{
    ar,
    interpreter::{ObjectInner, RuntimeError},
};

use super::{class::LoxClass, Object};

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
            return ar!(ObjectInner::Function(Arc::new(RwLock::new(
                method.bind(self.clone())
            ))));
        }
        throw(RuntimeError::new(
            name.clone(),
            format!("Undefined property '{}'.", name.lexeme,),
        ))
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
