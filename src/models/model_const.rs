use std::{collections::HashMap, sync::{Arc, Mutex}};

use super::{time::{Bound, ClockValue}, Label};

pub enum ConstValue {
    Int(i32),
    Float(f64),
    Time(ClockValue),
}

pub struct ConstStore {
    values : HashMap<Label, ConstValue>
}