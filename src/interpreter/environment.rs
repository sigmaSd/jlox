use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::{interpreter::Object, scanner::Token};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Option<Object>>,
    enclosing: Option<Arc<RwLock<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: Option<Arc<RwLock<Environment>>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing,
        }
    }
    pub fn define(&mut self, name: String, value: Option<Object>) {
        self.values.insert(name, value);
    }
    pub fn get(&self, token: &Token) -> Object {
        if let Some(Some(obj)) = self.values.get(&token.lexeme) {
            return obj.clone();
        }
        // search the enclosing env
        if let Some(obj) = self
            .enclosing
            .as_ref()
            .map(|enc| enc.try_read().unwrap().get(token))
        {
            return obj;
        }

        panic!("Undefined variable '{}'.", token.lexeme)
    }
    pub fn assign(&mut self, name: Token, value: Object) {
        if let std::collections::hash_map::Entry::Occupied(mut e) =
            self.values.entry(name.lexeme.clone())
        {
            e.insert(Some(value));
            return;
        }
        // search the enclosing env
        if let Some(enclosing) = self.enclosing.as_mut() {
            enclosing.try_write().unwrap().assign(name, value);
            return;
        }
        panic!("Undefined variable: '{}'", name.lexeme)
    }
}
