use std::fmt;
use std::ops::Add;
use std::ops::AddAssign;

use rand::distributions::Alphanumeric;
use rand::distributions::DistString;
use rand::Rng;
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

    pub fn set_domain(&self, domain : Label) -> Label {
        return domain + "." + &self.clone()
    }

    pub fn has_domain(&self, domain : &Label) -> bool {
        let prefix = domain.clone() + ".";
        return self.0.starts_with(&prefix.0)
    }

    pub fn new_random(rng : &mut impl Rng, len : usize) -> Self {
        Self::from(Alphanumeric.sample_string(rng, len))
    }

}

impl AsRef<str> for Label {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Default for Label {
    fn default() -> Self {
        Label::new()
    }
}

impl<T : ToString> Add<T> for Label {
    type Output = Label;
    fn add(self, rhs: T) -> Self::Output {
        Label::from(self.0 + &rhs.to_string())
    }
}

impl<T : ToString> AddAssign<T> for Label {
    fn add_assign(&mut self, rhs: T) {
        self.0 += &rhs.to_string()
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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
