use std::collections::HashSet;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, Not};
use std::cmp::min;

const CELL_SIZE : usize = 64;

// Structure for fast operations on boolean sets : And, Or, Not... Complexity O(n) to retrieve indexs after computation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BitSet {
    enabled: Vec<u64>
}
impl BitSet {

    pub fn new() -> Self {
        BitSet { enabled: Vec::new() }
    }

    pub fn single_bit(bit : usize) -> Self {
        let mut set = BitSet::new();
        set.enable(bit);
        set
    }

    pub fn get_indexs(bit : usize) -> (u64, usize) {
        let a_byte = 1 << (bit % CELL_SIZE);
        let byte_index = bit / CELL_SIZE;
        (a_byte, byte_index)
    }

    pub fn enable(&mut self, bit : usize) {
        let (new_byte, byte_index) = Self::get_indexs(bit);
        if byte_index >= self.enabled.len() {
            self.enabled.resize(byte_index + 1, 0);
        }
        self.enabled[byte_index] |= new_byte;
    }

    pub fn disable(&mut self, bit : usize) {
        let (new_byte, byte_index) = Self::get_indexs(bit);
        if byte_index >= self.enabled.len() {
            self.enabled.resize(byte_index + 1, 0);
        }
        self.enabled[byte_index] &= !new_byte;
        self.trim();
    }

    pub fn is_enabled(&self, bit : usize) -> bool {
        let (new_byte, byte_index) = Self::get_indexs(bit);
        if byte_index >= self.enabled.len() {
            false
        } else {
            (self.enabled[byte_index] & new_byte) > 0
        }
    }

    pub fn get_bits(&self) -> HashSet<usize> { // Might be optimized by unfolding
        let mut res : HashSet<usize> = HashSet::new();
        for (b_i,b) in self.enabled.iter().enumerate() { // Usually only one block, except if > 64 bits
            let mut rem = *b;
            let mut i : usize = 0;
            while rem > 0 {
                if rem % 2 == 1 {
                    res.insert(b_i * CELL_SIZE + i);
                }
                i += 1;
                rem >>= 1;
            }
        }
        res
    }

    pub fn get_newen(old : &BitSet, new : &BitSet) -> BitSet {
        let mut res = BitSet::new();
        for i in 0..new.enabled.len() {
            if old.enabled.len() <= i {
                res.enabled.push(new.enabled[i]);
            } else {
                let to_push = new.enabled[i] & (!old.enabled[i]);
                res.enabled.push(to_push);
            }
        }
        res
    }

    pub fn is_empty(&self) -> bool {
        for b in self.enabled.iter() {
            if *b != 0 {
                return false;
            }
        }
        return true;
    }

    pub fn covers(&self, other : &BitSet) -> bool {
        for i in 0..other.enabled.len() {
            if i >= self.enabled.len() { // && (other.enabled[i] > 0) Shouldn't be necessary because of trim
                return false
            }
            if (self.enabled[i] & other.enabled[i]) < other.enabled[i] {
                return false
            }
        }
        true
    }

    pub fn trim(&mut self) {
        while !self.enabled.is_empty() {
            if *self.enabled.last().unwrap() == 0 {
                self.enabled.pop();
            } else {
                break;
            }
        }
    }

}

impl BitOr for BitSet {
    type Output = BitSet;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        let (mut sink, source) = if self.enabled.len() >= rhs.enabled.len() {
            (self, rhs)
        } else {
            (rhs, self)
        };
        for i in 0..sink.enabled.len() {
            if i >= source.enabled.len() {
                break;
            }
            sink.enabled[i] |= source.enabled[i];
        }
        sink
    }
    
}

impl BitOrAssign for BitSet {

    fn bitor_assign(&mut self, rhs: Self) {
        if self.enabled.len() < rhs.enabled.len() {
            self.enabled.resize(rhs.enabled.len(), 0);
        }
        for (i,b) in rhs.enabled.into_iter().enumerate() {
            self.enabled[i] |= b;
        }
    }

}

impl BitAnd for BitSet {
    type Output = BitSet;
    
    fn bitand(self, rhs: Self) -> Self::Output {
        let (mut sink, source) = if self.enabled.len() <= rhs.enabled.len() {
            (self, rhs)
        } else {
            (rhs, self)
        };
        for i in 0..sink.enabled.len() {
            sink.enabled[i] &= source.enabled[i];
        }
        sink.trim();
        sink
    }
    
}

impl BitAndAssign for BitSet {

    fn bitand_assign(&mut self, rhs: Self) {
        let len = min(self.enabled.len(), rhs.enabled.len());
        if self.enabled.len() < len {
            self.enabled.resize(len, 0);
        }
        for i in 0..len {
            self.enabled[i] &= rhs.enabled[i];
        }
        self.trim();
    }

}

impl BitXor for BitSet {
    type Output = BitSet;
    
    fn bitxor(self, rhs: Self) -> Self::Output {
        let (mut sink, source) = if self.enabled.len() >= rhs.enabled.len() {
            (self, rhs)
        } else {
            (rhs, self)
        };
        for i in 0..sink.enabled.len() {
            if i >= source.enabled.len() {
                break;
            }
            sink.enabled[i] ^= source.enabled[i];
        }
        sink.trim();
        sink
    }
}

impl Not for BitSet {
    type Output = BitSet;

    fn not(self) -> Self::Output {
        let mut vec = self.enabled;
        vec.iter_mut().for_each(|x| *x = !*x);
        let mut set = BitSet::from(vec);
        set.trim();
        set
    }
}

impl From<Vec<u64>> for BitSet {
    fn from(value: Vec<u64>) -> Self {
        let mut set = BitSet { enabled: value };
        set.trim();
        set
    }
}

impl From<HashSet<usize>> for BitSet {
    fn from(value: HashSet<usize>) -> Self {
        let mut set = BitSet::new();
        for bit in value {
            set.enable(bit);
        }
        set
    }
}