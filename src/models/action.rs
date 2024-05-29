use std::fmt::Display;

use serde::{Deserialize, Serialize};

// Action enum :
// Epsilon : No label nor ID, used for internal invisible transitions

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    #[serde(rename = "_")]
    Epsilon,
    Internal(usize),
}

impl Default for Action {
    fn default() -> Self {
        Self::Epsilon
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Epsilon => write!(f, "_"),
            Self::Internal(i) => write!(f, "{}", i),
        }
    }
}