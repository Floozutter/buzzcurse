use std::hash::{Hash, Hasher};
use std::mem::discriminant;

#[derive(Clone, Copy, PartialEq)]
pub struct Botton(pub rdev::Button);

impl Eq for Botton {}

impl Hash for Botton {
    fn hash<H: Hasher>(&self, state: &mut H) {
        discriminant(&self.0).hash(state);
        if let rdev::Button::Unknown(u) = self.0 {
            u.hash(state);
        }
    }
}
