use crate::interpreter::LoxCallable;

use std::{
    fmt,
    sync::{Arc, RwLock},
};

pub mod class {
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
    }

    impl LoxClass {
        pub fn new(name: String, methods: HashMap<String, LoxFunction>) -> Self {
            Self { name, methods }
        }

        pub(crate) fn find_method(&self, name: &str) -> Option<LoxFunction> {
            self.methods.get(name).cloned()
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
}

#[derive(Clone)]
pub enum Object {
    Number(f64),
    String(String),
    Bool(bool),
    Function(Arc<RwLock<dyn LoxCallable>>),
    Class(class::LoxClass),
    Instance(class::LoxInstance),
    Null,
}
impl Default for Object {
    fn default() -> Self {
        Object::Null
    }
}
impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Instance(i1), Self::Instance(i2)) if i1.class.name == i2.class.name => true,
            (Self::Class(c1), Self::Class(c2)) if c1.name == c2.name => true,
            (Self::Number(n1), Self::Number(n2)) if n1 == n2 => true,
            (Self::String(s1), Self::String(s2)) if s1 == s2 => true,
            (Self::Bool(b1), Self::Bool(b2)) if b1 == b2 => true,
            (Self::Null, Self::Null) => true,
            (Self::Function(l0), Self::Function(r0)) => Arc::ptr_eq(l0, r0),
            _ => false,
        }
    }
}
impl Eq for Object {}
impl std::hash::Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}
impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            Self::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            Self::Bool(arg0) => f.debug_tuple("Bool").field(arg0).finish(),
            Self::Function(_) => f.debug_tuple("Function").finish(),
            Self::Class(c) => write!(f, "Class {}", c.to_string()),
            Self::Instance(i) => write!(f, "Instance {}", i.to_string()),
            Self::Null => write!(f, "Null"),
        }
    }
}
impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(arg0) => write!(f, "{}", arg0),

            Self::String(arg0) => write!(f, "{}", arg0),

            Self::Bool(arg0) => write!(f, "{}", arg0),
            Self::Null => write!(f, "Null"),
            Self::Class(c) => write!(f, "{}", c.to_string()),
            Self::Instance(i) => write!(f, "{}", i.to_string()),
            Self::Function(_) => write!(f, "<function>"),
        }
    }
}
impl Object {
    pub fn is_num(&self) -> bool {
        matches!(self, Self::Number(_))
    }
    pub fn is_str(&self) -> bool {
        matches!(self, Self::String(_))
    }
    pub fn is_fun(&self) -> bool {
        matches!(self, Self::Function(_) | Self::Class(_))
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
}

#[macro_export]
macro_rules! downcast_to_lox_callable {
    ($obj: expr) => {
        {
        let callable: Arc<RwLock<dyn LoxCallable>> =
        crate::try_downcast!($obj.clone() => Object::Function).unwrap_or_else(||

        Arc::new(RwLock::new(crate::try_downcast!($obj.clone() => Object::Class).unwrap()))

            );
        callable
        }
    };
}

#[macro_export]
macro_rules! downcast {
    ($obj: expr => $otype: path) => {
        crate::try_downcast!($obj => $otype).unwrap()
    };
}
#[macro_export]
macro_rules! try_downcast {
    ($obj: expr => $otype: path) => {
        if let $otype(obj) = $obj {
            Some(obj)
        } else {
            None
        }
    };
}
#[macro_export]
macro_rules! obj {
    ($obj: expr  ; $otype: path) => {
        $otype($obj)
    };
    ($obj: expr ; @rr $otype: path) => {
        $otype(std::sync::Arc::new(std::sync::RwLock::new($obj)))
    };
}
#[macro_export]
macro_rules! null_obj {
    () => {
        Object::Null
    };
}
