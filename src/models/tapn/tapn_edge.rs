use serde::{Deserialize, Serialize};

use crate::models::{time::TimeInterval, Edge};

use super::{TAPNPlace, TAPNTransition};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct TAPNEdgeData {
    pub interval : TimeInterval,
    pub weight : i32
}

pub type InputEdge = Edge<TAPNEdgeData, TAPNPlace, TAPNTransition>;
pub type OutputEdge = Edge<i32, TAPNTransition, TAPNPlace>;
pub type TransportEdge = Edge<TAPNEdgeData, TAPNPlace, TAPNPlace>;