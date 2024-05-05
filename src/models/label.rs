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

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn get_first_ident(&self) -> (Label, Label) {
        let split = self.0.split_once(".");
        match split {
            None => (self.clone(), Label::new()),
            Some((prefix, suffix)) => (Label::from(prefix), Label::from(suffix))
        }
    }

}

impl Default for Label {
    fn default() -> Self {
        Label::new()
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