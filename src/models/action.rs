use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Epsilon,
    Internal(usize, usize),
    Input(usize, usize),
    Output(usize, usize)
}