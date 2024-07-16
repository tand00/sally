use std::time::Instant;

use rand::{thread_rng, Rng};
use rayon::prelude::*;

use crate::models::markov::ProbabilisticChoice;
use crate::log::*;
use crate::models::{Model, ModelMaker};

pub trait Genetizable : Sync + Send {
    fn cross(&self, other : &Self) -> Self;
    fn mutate(&mut self);
}

pub struct GeneticOptimizer<T : Genetizable> {
    pub generator : Box<dyn (Fn() -> T) + Sync + Send>,
    pub fitness : Box<dyn (Fn(&T) -> f64) + Sync + Send>,
}

impl<T : Genetizable> GeneticOptimizer<T> {

    pub fn new<F,G>(generator : F, fitness : G) -> Self
        where 
            F : (Fn() -> T) + Sync + Send + 'static,
            G : (Fn(&T) -> f64) + Sync + Send + 'static
    {
        GeneticOptimizer {
            generator : Box::new(generator),
            fitness : Box::new(fitness),
        }
    }

    fn score_sort(&self, candidates : &mut Vec<(T, f64)>) {
        candidates.par_iter_mut().for_each(|x| {
            x.1 = (self.fitness)(&x.0)
        });
        candidates.par_sort_by(|a,b| {
            a.1.partial_cmp(&b.1).unwrap()
        });
    }

    pub fn generate_population(&self, population : usize) -> Vec<(T, f64)> {
        (0..population).into_par_iter().map(
            |_| ((self.generator)(), 0.0)
        ).collect()
    }

    pub fn optimize(&self, generations : usize, population : usize, elite : usize, mutation_rate : f64) -> (T, f64) {
        info("Genetic optimization");
        continue_info(format!("Generations : {generations} | Population : {population} | Elite size : {elite}"));
        let now = Instant::now();

        pending("Generating base population...");
        let mut candidates = self.generate_population(population);

        for g in 0..generations {
            pending(format!("Executing generation {}...", (g+1)));

            self.score_sort(&mut candidates);
            let best_score = candidates.last().unwrap().1;
            continue_info(format!("Best fitness : {best_score}"));

            if g == (generations - 1) {
                break;
            }

            let sampler = ProbabilisticChoice::new(candidates);
            let children_to_make = population - elite;
            let mut children : Vec<(T, f64)> = (0..children_to_make).into_par_iter().map(|_| {
                let p1 = sampler.sample();
                let p2 = sampler.sample();
                let mut child = p1.cross(p2);
                if thread_rng().gen::<f64>() < mutation_rate {
                    child.mutate();
                }
                (child, 0.0)
            }).collect();

            candidates = sampler.0;
            for _ in 0..elite {
                children.push(candidates.pop().unwrap());
            }
            candidates = children;
        }
        let time = now.elapsed().as_secs_f64();
        positive("Genetic optimization finished !");
        continue_info(format!("Time : {time}s"));
        candidates.pop().unwrap()
    }

}

impl<T : Genetizable + Model> GeneticOptimizer<T> {

    pub fn from_maker<M,F>(maker : M, fitness : F) -> Self
        where 
            M : ModelMaker<T> + 'static,
            F : (Fn(&T) -> f64) + Sync + Send + 'static
    {
        GeneticOptimizer {
            generator : Box::new(move || maker.make().0),
            fitness : Box::new(fitness),
        }
    }

}