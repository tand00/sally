use std::{collections::HashMap, sync::{Arc, Mutex}};

use super::{time::{ClockValue, Interval}, Label};

pub enum ConstValue {
    Int(i32),
    Float(f64),
    Time(ClockValue),
}

pub struct ConstStore {
    values : HashMap<Label, ConstValue>
}