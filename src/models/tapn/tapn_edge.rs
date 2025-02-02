use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::models::{time::TimeInterval, Edge};

use super::{TAPNPlace, TAPNTransition};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct TAPNEdgeData {
    pub interval : TimeInterval,
    pub weight : i32
}

impl Display for TAPNEdgeData {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : x{}", self.interval, self.weight)
    }

}

pub type InputEdge = Edge<TAPNEdgeData, TAPNPlace, TAPNTransition>;
pub type OutputEdge = Edge<i32, TAPNTransition, TAPNPlace>;
pub type TransportEdge = Edge<TAPNEdgeData, TAPNPlace, TAPNPlace>;