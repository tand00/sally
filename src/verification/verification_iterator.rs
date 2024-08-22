use std::collections::{HashMap, HashSet};

use crate::models::{Model, ModelState};

use super::{query::Query, EvaluationState};

pub trait VerificationIterator : Iterator {

    fn prepare<T : Model>(&mut self, query : Query, model : &T, initial : ModelState);

}