use std::{
    fmt,
    ops::Deref,
    sync::{Arc, RwLock},
};

use crate::uuid::Uuid;

use self::{class::LoxClass, instance::LoxInstance, lox_callable::LoxCallable};

pub mod class;
pub mod function;
mod instance;
pub mod lox_callable;

#[derive(Clone)]
pub enum ObjectInner {
    Number(f64),
    String(String),
    Bool(bool),
    Function(Arc<RwLock<dyn LoxCallable>>),
    Class(LoxClass),
    Instance(LoxInstance),
    Null,
}
#[derive(Clone)]
pub struct Object(pub ObjectInner, pub Uuid);

#[macro_export]
macro_rules! ar {
    ($e: expr) => {
        Object($e, crate::uuid::Uuid::new_v4())
    };
}
impl Default for Object {
    fn default() -> Self {
        ar!(ObjectInner::Null)
    }
}
impl Deref for Object {
    type Target = ObjectInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        let this: &ObjectInner = self;
        let other: &ObjectInner = other;

        match (this, other) {
            (ObjectInner::Instance(i1), ObjectInner::Instance(i2))
                if i1.class.name == i2.class.name =>
            {
                true
            }
            (ObjectInner::Class(c1), ObjectInner::Class(c2)) if c1.name == c2.name => true,
            (ObjectInner::Number(n1), ObjectInner::Number(n2)) if n1 == n2 => true,
            (ObjectInner::String(s1), ObjectInner::String(s2)) if s1 == s2 => true,
            (ObjectInner::Bool(b1), ObjectInner::Bool(b2)) if b1 == b2 => true,
            (ObjectInner::Null, ObjectInner::Null) => true,
            (ObjectInner::Function(l0), ObjectInner::Function(r0)) => Arc::ptr_eq(l0, r0),
            _ => false,
        }
    }
}
impl Eq for Object {}
impl std::hash::Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.1.hash(state)
    }
}
impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this: &ObjectInner = self;
        match this {
            ObjectInner::Number(arg0) => f.debug_tuple("Number").field(arg0).finish(),
            ObjectInner::String(arg0) => f.debug_tuple("String").field(arg0).finish(),
            ObjectInner::Bool(arg0) => f.debug_tuple("Bool").field(arg0).finish(),
            ObjectInner::Function(_) => f.debug_tuple("Function").finish(),
            ObjectInner::Class(c) => write!(f, "Class {}", c.to_string()),
            ObjectInner::Instance(i) => write!(f, "Instance {}", i.to_string()),
            ObjectInner::Null => write!(f, "nil"),
        }
    }
}
impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let this: &ObjectInner = self;
        match this {
            ObjectInner::Number(arg0) => write!(f, "{}", arg0),

            ObjectInner::String(arg0) => write!(f, "{}", arg0),

            ObjectInner::Bool(arg0) => write!(f, "{}", arg0),
            ObjectInner::Null => write!(f, "nil"),
            ObjectInner::Class(c) => write!(f, "{}", c.to_string()),
            ObjectInner::Instance(i) => write!(f, "{}", i.to_string()),
            ObjectInner::Function(lfn) => write!(f, "{}", lfn.try_read().unwrap()),
        }
    }
}
impl Object {
    pub fn is_num(&self) -> bool {
        matches!(self.0, ObjectInner::Number(_))
    }
    pub fn is_str(&self) -> bool {
        matches!(self.0, ObjectInner::String(_))
    }
    pub fn is_fun(&self) -> bool {
        // class also implements LoxCallable
        matches!(self.0, ObjectInner::Function(_) | ObjectInner::Class(_))
    }
    pub fn is_class(&self) -> bool {
        matches!(self.0, ObjectInner::Class(_))
    }
    pub fn is_null(&self) -> bool {
        matches!(self.0, ObjectInner::Null)
    }
}

#[macro_export]
macro_rules! downcast_to_lox_callable {
    ($obj: expr) => {
        {
        use crate::interpreter::object::lox_callable::LoxCallable;
        let callable: Arc<RwLock<dyn LoxCallable>> =
        crate::try_downcast!($obj.clone() => ObjectInner::Function).unwrap_or_else(||

        Arc::new(RwLock::new(crate::try_downcast!($obj.clone() => ObjectInner::Class).unwrap()))

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
        if let $otype(obj) = $obj.0 {
            Some(obj)
        } else {
            None
        }
    };
}
#[macro_export]
macro_rules! obj {
    ($obj: expr  ; $otype: path) => {
        crate::ar!($otype($obj))
    };
    ($obj: expr ; @rr $otype: path) => {
        crate::ar!($otype(std::sync::Arc::new(std::sync::RwLock::new($obj))))
    };
}
#[macro_export]
macro_rules! null_obj {
    () => {
        crate::ar!(crate::interpreter::ObjectInner::Null)
    };
}
