use std::ops::{BitAnd, BitOr, Not};
use std::cmp::min;


// Each action cell is a 64bit int, on 6bit processors should take the same time as bytes
const ACTION_CELL_SIZE : usize = 64; 

#[derive(Clone)]
pub struct ActionSet {
    enabled: Vec<u64>
}
impl ActionSet {

    pub fn new() -> Self {
        ActionSet { enabled: Vec::new() }
    }

    pub fn from(enabled : Vec<u64>) -> Self {
        ActionSet { enabled }
    }

    pub fn action_byte(action : usize) -> (u64, usize) {
        let a_byte = 1 << (action % ACTION_CELL_SIZE);
        let byte_index = action / ACTION_CELL_SIZE;
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
        let new_byte = !(1 << (action % ACTION_CELL_SIZE));
        let byte_index = action / ACTION_CELL_SIZE;
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

    pub fn merge(&mut self, other : &ActionSet) {
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
                    res.push(b_i * ACTION_CELL_SIZE + i);
                }
                i += 1;
                rem >>= 1;
            }
        }
        res
    }

    pub fn get_newen(old : &ActionSet, new : &ActionSet) -> ActionSet {
        let mut res = ActionSet::new();
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

impl BitOr for ActionSet {
    type Output = ActionSet;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut res = self.clone();
        res.merge(&rhs);
        res
    }
    
}

impl BitOr for &ActionSet {
    type Output = ActionSet;
    
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut res = self.clone();
        res.merge(rhs);
        res
    }
    
}

impl BitAnd for ActionSet {
    type Output = ActionSet;
    
    fn bitand(self, rhs: Self) -> Self::Output {
        let len = min(self.enabled.len(), rhs.enabled.len());
        let mut res : Vec<u64>= Vec::new();
        for i in 0..len {
            let byte = self.enabled[i] & rhs.enabled[i];
            res.push(byte);
        }
        ActionSet::from(res)
    }
    
}

impl BitAnd for &ActionSet {
    type Output = ActionSet;
    
    fn bitand(self, rhs: Self) -> Self::Output {
        let len = min(self.enabled.len(), rhs.enabled.len());
        let mut res : Vec<u64>= Vec::new();
        for i in 0..len {
            let byte = self.enabled[i] & rhs.enabled[i];
            res.push(byte);
        }
        ActionSet::from(res)
    }
    
}

impl Not for ActionSet {
    type Output = ActionSet;

    fn not(self) -> Self::Output {
        let mut res : Vec<u64> = Vec::new();
        for i in self.enabled {
            res.push(!i);
        }
        ActionSet::from(res)
    }
}

impl Not for &ActionSet {
    type Output = ActionSet;

    fn not(self) -> Self::Output {
        let mut res : Vec<u64> = Vec::new();
        for i in self.enabled.iter() {
            res.push(!(*i));
        }
        ActionSet::from(res)
    }
}