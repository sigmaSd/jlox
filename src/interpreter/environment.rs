use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{interpreter::Object, scanner::Token};

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Option<Object>>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new(enclosing: Option<Rc<RefCell<Environment>>>) -> Self {
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
        if let Some(obj) = self.enclosing.as_ref().map(|enc| enc.borrow().get(token)) {
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
            enclosing.borrow_mut().assign(name, value);
            return;
        }
        panic!("Undefined variable: '{}'", name.lexeme)
    }
}
