use std::{marker::PhantomData, usize};

pub trait FixPoint<T : PartialEq> {

    fn next_step(&mut self, x : &T) -> T;

    fn converge(&mut self, x0 : &T, max_n : usize) -> Option<(usize,T)> {
        let mut item1 = self.next_step(x0);
        if item1 == (*x0) {
            return Some((0, item1));
        }
        let mut item2 = self.next_step(&item1);
        for i in 0..max_n {
            if item1 == item2 {
                return Some((i + 1, item1));
            }
            std::mem::swap(&mut item1, &mut item2);
            item2 = self.next_step(&item1);
        }
        None
    }

    fn get_fix_point(&mut self, x0 : &T) -> T {
        self.converge(x0, usize::MAX).unwrap().1
    }

}

pub struct FSequence<T : PartialEq, F : Fn(&T) -> T> {
    pub recurrence : F,
    pub phantom : PhantomData<T>
}
impl<T : PartialEq, F : Fn(&T) -> T> FSequence<T, F> {

    pub fn new(recurrence : F) -> Self {
        FSequence { recurrence, phantom: PhantomData }
    }

    pub fn generate_n(&self, x0 : T, n : usize) -> Vec<T> {
        let mut res = Vec::new();
        res.reserve(n+1);
        res[0] = x0;
        for i in 0..n {
            res[i+1] = (self.recurrence)(&res[i]);
        }
        res
    }

}
impl<T : PartialEq, F : Fn(&T) -> T> FixPoint<T> for FSequence<T,F> {
    fn next_step(&mut self, x : &T) -> T {
        (self.recurrence)(x)
    }
}
