use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelStorage {
    EmptyStorage,
    Integer(i32),
    Float(f64),
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
            Float(f) => f as i32,
            _ => panic!("Incorrect storage structure")
        }
    }

    pub fn float(self) -> f64 {
        match self {
            Float(f) => f,
            Integer(i) => i as f64,
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

    pub fn mut_float(&mut self) -> &mut f64 {
        match self {
            Float(f) => f,
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

    pub fn ref_float(&self) -> &f64 {
        match self {
            Float(f) => f,
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

    pub fn is_float(&self) -> bool {
        match self {
            Float(_) => true,
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

impl From<f64> for ModelStorage {
    fn from(value: f64) -> Self {
        Float(value)
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

impl Hash for ModelStorage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            EmptyStorage => state.write_u8(0),
            Integer(i) => { 
                state.write_u8(1);
                i.hash(state);
            },
            Float(f) => { 
                state.write_u8(2);
                ((*f * 100_000_000.0) as u64).hash(state);
            },
            Tuple(a, b) => {
                state.write_u8(3);
                a.hash(state);
                b.hash(state);
            },
            Vector(v) => {
                state.write_u8(4);
                v.hash(state);
            },
            
        }
    }
}

impl PartialEq for ModelStorage {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (EmptyStorage, EmptyStorage) => true,
            (Integer(i1), Integer(i2)) => i1 == i2,
            (Float(f1), Float(f2)) => 
                (f1 == f2) || (f1.is_nan() && f2.is_nan()),
            (Vector(v1), Vector(v2)) => v1 == v2,
            (Tuple(a,b), Tuple(c,d)) => (a,b) == (c,d),
            _ => false
        }
    }
}

impl Eq for ModelStorage { }