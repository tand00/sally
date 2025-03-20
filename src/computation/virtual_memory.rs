use std::{cmp::min, fmt::Display, hash::{DefaultHasher, Hash, Hasher}, mem::size_of};

use serde::{Deserialize, Serialize};

use crate::models::model_var::{ModelVar, VarType};

use VarType::*;

pub type EvaluationType = i32;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct VirtualMemory {
    storage : Vec<u8>
}

impl VirtualMemory {

    pub fn new() -> VirtualMemory {
        VirtualMemory { storage : Vec::new() }
    }

    pub fn from_size(size : usize) -> VirtualMemory {
        VirtualMemory { storage : vec![0 ; size] }
    }

    pub fn evaluate_at<T : Copy>(&self, address : usize) -> T {
        if address + size_of::<T>() > self.len() {
            panic!("Pointer out of bound !")
        }
        let storage = self.storage.as_ptr();
        let value : T;
        unsafe {
            let var_ptr = storage.add(address) as *const T;
            value = *var_ptr;
        }
        value
    }

    pub fn set_at<T : Copy>(&mut self, address : usize, value : T) {
        let type_size = size_of::<T>();
        if address + type_size > self.len() {
            panic!("Pointer out of bound !")
        }
        let storage = self.storage.as_mut_ptr();
        unsafe {
            let var_ptr = storage.add(address) as *mut T;
            *var_ptr = value;
        }
    }

    pub fn evaluate(&self, var : &ModelVar) -> EvaluationType {
        if !var.is_mapped() || (var.get_address() + var.size() > self.len()) {
            panic!("Pointer out of bound !")
        }
        let address = var.get_address();
        match var.get_type() {
            VarU8 => self.evaluate_at::<u8>(address) as EvaluationType,
            VarI8 => self.evaluate_at::<i8>(address) as EvaluationType,
            VarU16 => self.evaluate_at::<u16>(address) as EvaluationType,
            VarI16 => self.evaluate_at::<i16>(address) as EvaluationType,
            VarU32 => self.evaluate_at::<u32>(address) as EvaluationType,
            VarI32 => self.evaluate_at::<i32>(address) as EvaluationType,
            _ => panic!("Can't evaluate untyped var !")
        }
    }

    pub fn set(&mut self, var : &ModelVar, value : EvaluationType) {
        if !var.is_mapped() || (var.get_address() + var.size() > self.len()) {
            panic!("Pointer out of bound !")
        }
        let address = var.get_address();
        match var.get_type() {
            VarU8 => self.set_at::<u8>(address, value as u8),
            VarI8 => self.set_at::<i8>(address, value as i8),
            VarU16 => self.set_at::<u16>(address, value as u16),
            VarI16 => self.set_at::<i16>(address, value as i16),
            VarU32 => self.set_at::<u32>(address, value as u32),
            VarI32 => self.set_at::<i32>(address, value as i32),
            _ => panic!("Can't set untypes var !")
        }
    }

    pub fn len(&self) -> usize {
        self.storage.len()
    }

    pub fn is_empty(&self) -> bool {
        self.storage.is_empty()
    }

    pub fn define(&mut self, var : &mut ModelVar, var_type : VarType) {
        if var.is_mapped() {
            panic!("Can't redefine already mapped var !");
        }
        var.set_type(var_type);
        var.set_address(self.len());
        self.storage.resize(self.len() + var.size(), 0);
    }

    pub fn copy_from(&mut self, other : &VirtualMemory) {
        let to_copy = min(other.len(), self.len());
        self.storage[0..to_copy].copy_from_slice(&other.storage[0..to_copy])
    }

    pub fn resize(&mut self, size : usize) {
        self.storage.resize(size, 0)
    }

    pub fn size_delta(&mut self, delta : usize) {
        self.storage.resize(self.len() + delta, 0)
    }

    pub fn get_hash(&self) -> u64 {
        let mut s = DefaultHasher::new();
        self.hash(&mut s);
        s.finish()
    }

}

impl Display for VirtualMemory {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return write!(f, "VirtualMemory[EMPTY]");
        }
        write!(f, "VirtualMemory[")?;
        for cursor in 0..self.len() {
            if cursor % 16 == 0 {
                write!(f, "\n{:x} |\t", cursor)?;
            }
            let value = self.storage[cursor];
            write!(f, "{:x} ", value)?;
        }
        write!(f, "\n]")
    }

}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct VariableDefiner {
    size : usize
}

impl VariableDefiner {

    pub fn new() -> VariableDefiner {
        VariableDefiner { size : 0 }
    }

    pub fn define(&mut self, var : &mut ModelVar, var_type : VarType) {
        if var.is_mapped() {
            panic!("Can't redefine already mapped var !");
        }
        var.set_type(var_type);
        var.set_address(self.size);
        self.size += var.size();
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn append(&mut self, other : VariableDefiner) {
        self.size += other.size()
    }

    pub fn clear(&mut self) {
        self.size = 0;
    }

}

impl From<VariableDefiner> for VirtualMemory {

    fn from(definer : VariableDefiner) -> Self {
        VirtualMemory::from_size(definer.size())
    }

}
