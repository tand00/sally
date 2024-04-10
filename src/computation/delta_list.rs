use std::collections::HashMap;
use std::ops::Add;
use std::cmp::PartialOrd;

#[derive(Debug, Clone)]
pub struct DeltaList<T> {
    elements: HashMap<usize,T>,
    delta: T,
    index_min: Vec<usize>
}

impl<T : Add<Output = T> + PartialOrd + Copy> DeltaList<T> {

    pub fn new(delta: T) -> Self {
        DeltaList {
            elements: HashMap::new(),
            delta,
            index_min: Vec::new()
        }
    }

    pub fn from(source : HashMap<usize,T>, delta: T) -> Self {
        let mut list = Self::new(delta);
        list.elements = source;
        list.refresh_min();
        list
    }

    pub fn push(&mut self, index : usize, x : T) {
        if self.elements.is_empty() {
            self.elements.insert(index, x + self.delta);
            self.index_min = vec![0];
            return;
        }
        if x < self.elements[&self.index_min[0]] {
            self.index_min = vec![self.elements.len()];
        }
        if x == self.elements[&self.index_min[0]] {
            self.index_min.push(self.elements.len());
        }
        self.elements.insert(index, x);
    }

    pub fn delta(&mut self, dx : T) {
        self.delta = self.delta + dx;
    }

    pub fn at(&self, i : usize) -> T {
        self.elements[&i] + self.delta
    }

    pub fn remove(&mut self, i : usize) -> T {
        let pos_min = self.index_min.iter().position(|x| *x == i);
        if pos_min.is_some() {
            let pos_min = pos_min.unwrap();
            self.index_min.remove(pos_min);
            if self.index_min.is_empty() {
                self.refresh_min()
            }
        }
        self.elements.remove(&i).unwrap()
    }

    pub fn index_min(&self) -> Vec<usize> {
        self.index_min.clone()
    }

    pub fn min_value(&self) -> T {
        if self.elements.is_empty() {
            panic!("DeltaList is empty, no min !")
        }
        self.at(self.index_min()[0])
    }

    fn refresh_min(&mut self) {
        if self.elements.is_empty() {
            return;
        }
        let mut min_value = *self.elements.values().next().unwrap();
        self.index_min = Vec::new();
        for (i,x) in self.elements.iter() {
            if *x == min_value {
                self.index_min.push(*i);
            } else if *x < min_value {
                self.index_min = vec![*i];
                min_value = *x;
            }
        }
    }

    pub fn contains(&self, key : &usize) -> bool {
        self.elements.contains_key(key)
    }

    pub fn merge(&mut self, other : DeltaList<T>) {
        for (k,x) in other.elements {
            
        }
    }

}