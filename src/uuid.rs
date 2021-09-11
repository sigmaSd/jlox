use std::time;

#[derive(Clone, Hash)]
pub struct Uuid(time::Instant);
impl Uuid {
    pub fn new_v4() -> Self {
        Self(time::Instant::now())
    }
}
