mod bit_set;
mod dbm;
mod disjoint_interval;

pub mod virtual_memory;

pub use bit_set::BitSet;
pub use dbm::DBM;
pub use disjoint_interval::DisjointInterval;

#[macro_export]
macro_rules! flag {
    ($n:literal) => {
        1 << $n
    };
}