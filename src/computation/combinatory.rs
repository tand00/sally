/// Returns every combination of K elements of a slice, keeping the relative order
pub struct KInVec<'a, T> {
    vec: &'a [T],
    chosen: Vec<usize>,
}

impl<'a, T> KInVec<'a, T> {
    pub fn of(k: usize, value: &'a [T]) -> Self {
        KInVec {
            vec: value,
            chosen: (0..k).collect(),
        }
    }
    #[inline]
    pub fn k(&self) -> usize {
        self.chosen.len()
    }
}

impl<'a, T> Iterator for KInVec<'a, T> {

    type Item = Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        let n = self.vec.len();
        let k = self.k();
        let last_i = k - 1;

        if k > n || self.chosen[last_i] >= n {
            return None;
        }

        let res = Some(self.chosen.iter().map(|i| &self.vec[*i]).collect());
        self.chosen[last_i] += 1;
        if self.chosen[last_i] == n && self.chosen[0] < n - k {
            let mut pending = 1;
            let mut to_move = last_i - pending;
            self.chosen[to_move] += 1;
            while (self.chosen[to_move] == n - pending) && (to_move > 0) {
                pending += 1;
                to_move -= 1;
                self.chosen[to_move] += 1;
            }
            for i in 0..pending {
                self.chosen[to_move + i + 1] = self.chosen[to_move + i] + 1;
            }
        }
        res
    }

}

pub struct CartesianProduct<'a, T> {
    vecs : &'a [Vec<T>],
    chosen : Vec<usize>
}

impl<'a, T> CartesianProduct<'a, T> {

    pub fn of(value: &'a [Vec<T>]) -> Self {
        CartesianProduct {
            vecs : value,
            chosen : vec![0 ; value.len()]
        }
    }

}

impl<'a, T> Iterator for CartesianProduct<'a, T> {

    type Item = Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        let n_vecs = self.vecs.len();
        let last_i = n_vecs - 1;

        if self.chosen[0] == self.vecs[0].len() {
            return None;
        }

        let res = self.chosen.iter().enumerate().map(|(i, x)| {
            &self.vecs[i][*x]
        }).collect();
        self.chosen[last_i] += 1;

        if self.chosen[last_i] == self.vecs[last_i].len() && self.chosen[0] < self.vecs[0].len() {
            let mut to_move = last_i;
            while self.chosen[to_move] == self.vecs[to_move].len() && to_move > 0 {
                self.chosen[to_move] = 0;
                to_move -= 1;
                self.chosen[to_move] += 1;
            }
        }

        Some(res)
    }

}

pub struct Parts<'a, T> {
    vec : &'a [T],
    intern_iter : KInVec<'a, T>
}

impl<'a, T> Parts<'a, T> {
    pub fn of(value: &'a [T]) -> Self {
        Parts {
            vec : value,
            intern_iter : KInVec::of(1, value)
        }
    }
}

impl<'a, T> Iterator for Parts<'a, T> {

    type Item = Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        let k = self.intern_iter.k();
        let mut next_part = self.intern_iter.next();
        if next_part.is_none() {
            self.intern_iter = KInVec::of(k + 1, &self.vec);
            next_part = self.intern_iter.next();
        }
        next_part
    }

}
