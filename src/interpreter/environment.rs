use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::{interpreter::Object, scanner::Token};

#[derive(Debug, Clone)]
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
    pub fn get_at(&self, distance: &usize, name: &str) -> Object {
        self.ancestor(distance)
            .try_read()
            .unwrap()
            .values
            .get(name)
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
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
        eprintln!("Undefined variable: '{}'", name.lexeme)
    }

    fn ancestor(&self, distance: &usize) -> Arc<RwLock<Environment>> {
        let mut environment = Arc::new(RwLock::new(self.clone()));
        for _ in 0..*distance {
            environment = environment
                .clone()
                .try_read()
                .unwrap()
                .enclosing
                .as_ref()
                .unwrap()
                .clone();
        }
        environment
    }
    pub fn assign_at(&mut self, distance: &usize, name: Token, value: Object) {
        self.ancestor(distance)
            .try_write()
            .unwrap()
            .values
            .insert(name.lexeme, Some(value));
    }
}
