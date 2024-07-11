use rand::{distributions::{Distribution, WeightedIndex}, thread_rng};

pub mod markov_node;
pub mod markov_chain;

#[derive(Debug, Clone)]
pub struct ProbabilisticChoice<T>(pub Vec<(T, f64)>);

impl<T> ProbabilisticChoice<T> {

    pub fn normalized(&self) -> Self
        where T : Clone
    {
        let sum : f64 = self.0.iter().map(|x| x.1).sum();
        Self(self.0.iter().map(|x| {
            (x.0.clone(), x.1 / sum)
        }).collect())
    }

    pub fn sample(&self) -> &T {
        let dist = WeightedIndex::new(self.0.iter().map(|x| x.1)).unwrap();
        let mut rng = thread_rng();
        let sample = dist.sample(&mut rng);
        &self.0[sample].0
    }

}