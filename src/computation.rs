mod bit_set;
mod dbm;

pub mod virtual_memory;
pub mod combinatory;
pub mod intervals;

pub use bit_set::BitSet;
pub use dbm::DBM;

#[macro_export]
macro_rules! flag {
    ($n:literal) => {
        1 << $n
    };
}