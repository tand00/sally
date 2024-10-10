use std::usize;

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

pub struct IntSequence<F : Fn(i32) -> i32> {
    pub recurrence : F
}
impl<F : Fn(i32) -> i32> IntSequence<F> {

    pub fn new(recurrence : F) -> Self {
        IntSequence { recurrence }
    }

} 
impl<F : Fn(i32) -> i32> FixPoint<i32> for IntSequence<F> {
    fn next_step(&mut self, x : &i32) -> i32 {
        (self.recurrence)(*x)
    }
}