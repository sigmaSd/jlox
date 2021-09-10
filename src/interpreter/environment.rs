use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use trycatch::throw;

use crate::{interpreter::Object, scanner::Token};

#[derive(Debug, Clone)]
pub struct Environment {
    values: Arc<RwLock<HashMap<String, Option<Object>>>>,
    pub enclosing: Option<Arc<RwLock<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: Option<Arc<RwLock<Environment>>>) -> Self {
        Self {
            values: Default::default(),
            enclosing,
        }
    }
    pub fn define(&mut self, name: String, value: Option<Object>) {
        self.values.try_write().unwrap().insert(name, value);
    }
    pub fn get(&self, token: &Token) -> Object {
        if let Some(Some(obj)) = self.values.try_read().unwrap().get(&token.lexeme) {
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

        throw(format!(
            "Undefined variable '{}'.\n[line {}]",
            token.lexeme, token.line
        ))
    }
    pub fn get_at(&self, distance: &usize, name: &str) -> Object {
        self.ancestor(distance)
            .try_read()
            .unwrap()
            .values
            .try_read()
            .unwrap()
            .get(name)
            .as_ref()
            .unwrap()
            .as_ref()
            .unwrap()
            .clone()
    }
    pub fn assign(&mut self, name: Token, value: Object) {
        if let std::collections::hash_map::Entry::Occupied(mut e) =
            self.values.try_write().unwrap().entry(name.lexeme.clone())
        {
            e.insert(Some(value));
            return;
        }
        // search the enclosing env
        if let Some(enclosing) = self.enclosing.as_mut() {
            enclosing.try_write().unwrap().assign(name, value);
            return;
        }
        throw(format!(
            "Undefined variable '{}'.\n[line {}]",
            name.lexeme, name.line
        ))
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
            .try_write()
            .unwrap()
            .insert(name.lexeme, Some(value));
    }
}
