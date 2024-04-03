use crate::models::time::TimeBound;

pub struct DBM {
    size : usize,
    constraints : Vec<TimeBound>,
}

impl DBM {
    pub fn new(size : usize) -> DBM {
        let mut matrix = DBM {
            size,
            constraints: Vec::with_capacity(size * size)
        };
        matrix.constraints.resize(size * size, TimeBound::Large(0));
        matrix
    }
}