use rand::{distributions::{Distribution, WeightedIndex}, thread_rng};

pub mod markov_node;
pub mod markov_chain;

#[derive(Debug, Clone)]
pub struct ProbabilisticChoice<T>(pub Vec<(T, f64)>, WeightedIndex<f64>);

impl<T> ProbabilisticChoice<T> {

    pub fn new(values : Vec<(T, f64)>) -> Self {
        let dist = WeightedIndex::new(values.iter().map(|x| x.1)).unwrap();
        ProbabilisticChoice(values, dist)
    }

    pub fn normalized(&self) -> Self
        where T : Clone
    {
        let sum : f64 = self.0.iter().map(|x| x.1).sum();
        Self::new(self.0.iter().map(|x| {
            (x.0.clone(), x.1 / sum)
        }).collect())
    }

    pub fn sample(&self) -> &T {
        let dist = &self.1;
        let mut rng = thread_rng();
        let sample = dist.sample(&mut rng);
        &self.0[sample].0
    }

}