use std::ops::{BitAnd, BitOr, Not};
use std::cmp::min;


// Each cell is a 64bit int, on 64bit processors should take the same time as bytes
const CELL_SIZE : usize = 64; 

// Structure for fast operations on boolean sets : And, Or, Not... Complexity O(n) to retrieve indexs after computation
#[derive(Clone)]
pub struct BitSet {
    enabled: Vec<u64>
}
impl BitSet {

    pub fn new() -> Self {
        BitSet { enabled: Vec::new() }
    }

    pub fn from(enabled : Vec<u64>) -> Self {
        BitSet { enabled }
    }

    pub fn action_byte(action : usize) -> (u64, usize) {
        let a_byte = 1 << (action % CELL_SIZE);
        let byte_index = action / CELL_SIZE;
        (a_byte, byte_index)
    }

    pub fn enable(&mut self, action : usize) {
        let (new_byte, byte_index) = Self::action_byte(action);
        if byte_index >= self.enabled.len() {
            self.enabled.resize(byte_index + 1, 0);
        }
        self.enabled[byte_index] |= new_byte;
    }

    pub fn disable(&mut self, action : usize) {
        let new_byte = !(1 << (action % CELL_SIZE));
        let byte_index = action / CELL_SIZE;
        if byte_index >= self.enabled.len() {
            self.enabled.resize(byte_index + 1, 0);
        }
        self.enabled[byte_index] &= new_byte;
    }

    pub fn is_enabled(&self, action : usize) -> bool {
        let (new_byte, byte_index) = Self::action_byte(action);
        if byte_index >= self.enabled.len() {
            false
        } else {
            (self.enabled[byte_index] & new_byte) > 0
        }
    }

    pub fn merge(&mut self, other : &BitSet) {
        if self.enabled.len() < other.enabled.len() {
            self.enabled.resize(other.enabled.len(), 0);
        }
        for (i,b) in other.enabled.iter().enumerate() {
            self.enabled[i] |= b;
        }
    }

    pub fn get_actions(&self) -> Vec<usize> {
        let mut res : Vec<usize> = Vec::new();
        for (b_i,b) in self.enabled.iter().enumerate() { // Usually only one block, except if > 64 actions
            let mut rem = *b;
            let mut i : usize = 0;
            while rem > 0 {
                if rem % 2 == 1 {
                    res.push(b_i * CELL_SIZE + i);
                }
                i += 1;
                rem >>= 1;
            }
        }
        res
    }

    pub fn get_newen(old : &BitSet, new : &BitSet) -> BitSet {
        let mut res = BitSet::new();
        let mut i : usize = 0;
        while i < new.enabled.len() {
            if old.enabled.len() <= i {
                res.enabled.push(new.enabled[i]);
            } else {
                let to_push = new.enabled[i] & (!old.enabled[i]);
                res.enabled.push(to_push);
            }
            i += 1;
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

}

impl BitOr for BitSet {
    type Output = BitSet;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut res = self.clone();
        res.merge(&rhs);
        res
    }
    
}

impl BitOr for &BitSet {
    type Output = BitSet;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut res = self.clone();
        res.merge(rhs);
        res
    }
    
}

impl BitAnd for BitSet {
    type Output = BitSet;
    
    fn bitand(self, rhs: Self) -> Self::Output {
        let len = min(self.enabled.len(), rhs.enabled.len());
        let mut res : Vec<u64>= Vec::new();
        for i in 0..len {
            let byte = self.enabled[i] & rhs.enabled[i];
            res.push(byte);
        }
        BitSet::from(res)
    }
    
}

impl BitAnd for &BitSet {
    type Output = BitSet;
    
    fn bitand(self, rhs: Self) -> Self::Output {
        let len = min(self.enabled.len(), rhs.enabled.len());
        let mut res : Vec<u64>= Vec::new();
        for i in 0..len {
            let byte = self.enabled[i] & rhs.enabled[i];
            res.push(byte);
        }
        BitSet::from(res)
    }
    
}

impl Not for BitSet {
    type Output = BitSet;

    fn not(self) -> Self::Output {
        let mut res : Vec<u64> = Vec::new();
        for i in self.enabled {
            res.push(!i);
        }
        BitSet::from(res)
    }
}

impl Not for &BitSet {
    type Output = BitSet;

    fn not(self) -> Self::Output {
        let mut res : Vec<u64> = Vec::new();
        for i in self.enabled.iter() {
            res.push(!(*i));
        }
        BitSet::from(res)
    }
}