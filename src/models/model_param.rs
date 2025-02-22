use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{time::{TimeBound, TimeInterval}, Label};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelParam {
    IntParam(i32),
    FloatParam(f64),
    TimeIntervalParam(TimeInterval),
    TimeBoundParam(TimeBound),
    StringParam(String)
}

pub type NamedParams = HashMap<Label, ModelParam>;

pub enum ParamsSet {
    GeneralParams(NamedParams),
    NodeParams(Label, NamedParams),
    EdgeParams(Label, Label, NamedParams),
}

pub type ModelParams = Vec<ParamsSet>;
