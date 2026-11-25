use std::hash::Hash;

use serde::{Deserialize, Serialize};

use crate::real::Real;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModelStorage {
    EmptyStorage,
    Integer(i32),
    Real(Real),
    Bin(u64),
    Vector(Vec<ModelStorage>),
    Tuple(Box<ModelStorage>, Box<ModelStorage>)
}

use ModelStorage::*;

impl ModelStorage {

    pub fn is_empty(&self) -> bool {
        *self == EmptyStorage
    }

    pub fn int(self) -> i32 {
        match self {
            Integer(i) => i,
            Real(f) => f.value(),
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn real(self) -> Real {
        match self {
            Real(f) => f,
            Integer(i) => i.into(),
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn bin(self) -> u64 {
        match self {
            Integer(i) => i as u64,
            Bin(u) => u, 
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn tuple(self) -> (ModelStorage, ModelStorage) {
        match self {
            Tuple(a,b) => (*a, *b),
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn vec(self) -> Vec<ModelStorage> {
        match self {
            Vector(v) => v,
            Tuple(a, b) => vec![*a,*b],
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn mut_int(&mut self) -> &mut i32 {
        match self {
            Integer(i) => i,
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn mut_real(&mut self) -> &mut Real {
        match self {
            Real(f) => f,
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn mut_bin(&mut self) -> &mut u64 {
        match self {
            Bin(u) => u,
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn mut_tuple(&mut self) -> (&mut ModelStorage, &mut ModelStorage) {
        match self {
            Tuple(a,b) => (a, b),
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn mut_vec(&mut self) -> &mut Vec<ModelStorage> {
        match self {
            Vector(v) => v,
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn ref_int(&self) -> &i32 {
        match self {
            Integer(i) => i,
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn ref_real(&self) -> &Real {
        match self {
            Real(f) => f,
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn ref_bin(&self) -> &u64 {
        match self {
            Bin(u) => u,
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn ref_tuple(&self) -> (&ModelStorage, &ModelStorage) {
        match self {
            Tuple(a,b) => (a, b),
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn ref_vec(&self) -> &Vec<ModelStorage> {
        match self {
            Vector(v) => v,
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn is_int(&self) -> bool {
        match self {
            Integer(_) => true,
            _ => false
        }
    }

    pub fn is_real(&self) -> bool {
        match self {
            Real(_) => true,
            _ => false
        }
    }

    pub fn is_bin(&self) -> bool {
        match self {
            Bin(_) => true,
            _ => false
        }
    }

    pub fn is_tuple(&self) -> bool {
        match self {
            Tuple(_,_) => true,
            _ => false
        }
    }

    pub fn is_vec(&self) -> bool {
        match self {
            Vector(_) => true,
            _ => false
        }
    }

}

impl Default for ModelStorage {
    fn default() -> Self {
        EmptyStorage
    }
}

impl From<i32> for ModelStorage {
    fn from(value: i32) -> Self {
        Integer(value)
    }
}

impl From<Real> for ModelStorage {
    fn from(value: Real) -> Self {
        Real(value)
    }
}

impl From<u64> for ModelStorage {
    fn from(value: u64) -> Self {
        Bin(value)
    }
}

impl From<(ModelStorage, ModelStorage)> for ModelStorage {
    fn from(value: (ModelStorage, ModelStorage)) -> Self {
        Tuple(Box::new(value.0), Box::new(value.1))
    }
}

impl From<Vec<ModelStorage>> for ModelStorage {
    fn from(value: Vec<ModelStorage>) -> Self {
        Vector(value)
    }
}