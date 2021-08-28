use crate::interpreter::LoxCallable;

use std::{
    fmt,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub enum Object {
    Number(f64),
    String(String),
    Bool(bool),
    Function(Arc<RwLock<dyn LoxCallable>>),
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
        matches!(self, Self::Function(_))
    }
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
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
