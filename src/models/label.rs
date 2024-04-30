use std::fmt;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(String);

/// Wrapper of String to be used in model definitions (transitions and states labels...)
impl Label {
    pub fn new() -> Self {
        Label(String::new())
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'", self.0)
    }
}

/// Short label constructor
pub fn lbl(s : &str) -> Label {
    Label::from(s)
}

impl<T : Into<String>> From<T> for Label {
    fn from(value: T) -> Self {
        Label(value.into())
    }
}