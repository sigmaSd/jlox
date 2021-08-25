use std::str::FromStr;

#[macro_export]
macro_rules! obj {
    ($e: expr) => {
        (Object($e.to_string()))
    };
}

#[derive(Clone, Debug)]
pub struct Object(pub String);
impl Object {
    pub fn downcast<T: FromStr>(&self) -> T {
        self.try_downcast().unwrap()
    }
    pub fn try_downcast<T: FromStr>(&self) -> Option<T> {
        self.0.parse().ok()
    }
    pub fn is<T: FromStr>(&self) -> bool {
        self.try_downcast::<T>().is_some()
    }
}
