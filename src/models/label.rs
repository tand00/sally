use std::fmt;
use std::cmp;
use std::hash;

#[derive(Debug, Clone)]
pub struct Label(String);

/// Abstraction of String to be used in model definitions (transitions and states labels...)
impl Label {
    pub fn new() -> Self {
        Label(String::new())
    }
    pub fn from(lbl : &str) -> Self {
        Label(String::from(lbl))
    }
    pub fn from_string(str : impl ToString) -> Self {
        Label(str.to_string())
    }
}
impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'", self.0)
    }
}
impl cmp::PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl cmp::Eq for Label {}
impl hash::Hash for Label {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

/// Short label constructor
pub fn lbl(s : &str) -> Label {
    Label::from(s)
}