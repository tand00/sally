use std::{cmp::max, sync::Mutex, thread, time::Instant};

use crate::{models::{ModelContext, ModelVar, ModelObject, ModelState}, solution::SolverResult, verification::VerificationBound};
use crate::log::*;

#[derive(Debug, Clone)]
pub struct SMCMaxSeen {
    pub runs_needed : usize,
}

impl SMCMaxSeen {

    pub fn new(runs : usize) -> Self {
        SMCMaxSeen {
            runs_needed : runs,
        }
    }

    pub fn estimate_max(&self, model : &dyn ModelObject, ctx : &ModelContext, initial : &ModelState, bound : VerificationBound) -> SolverResult {
        info("Estimating max tokens using SMC...");
        continue_info(format!("Runs to be executed : {}", self.runs_needed));
        pending("Starting...");
        let now = Instant::now();
        let bound = bound.apply_to(ctx).unwrap();
        let mut max_seen = 0;
        for _ in 0..self.runs_needed {
            let iterator = model.random_run(initial, bound.clone());
            for (state, _, _) in iterator {
                let tokens = state.marking_sum(ctx.get_vars());
                if tokens > max_seen {
                    max_seen = tokens;
                }
            }
        }
        let elapsed = now.elapsed().as_secs_f64();
        positive(format!("Estimation complete, max seen : {}", max_seen));
        continue_info(format!("Time elapsed : {}s", elapsed));
        SolverResult::IntResult(max_seen)
    }

    pub fn parallel_estimate_max(&self, model : &dyn ModelObject, ctx : &ModelContext, initial : &ModelState, bound : VerificationBound) -> SolverResult {
        info("Estimating max tokens using SMC...");

        let threads = thread::available_parallelism().unwrap().get();
        continue_info(format!("Parallel mode [Threads : {}]", threads));
        continue_info(format!("Runs to be executed : {}", self.runs_needed));
        pending("Starting...");
        let now = Instant::now();

        let bound = bound.apply_to(ctx).unwrap();
        let runs_done : Mutex<usize> = Mutex::new(0);
        //let vars : Vec<ModelVar> = ctx.get_vars().map(ModelVar::clone).collect();

        let max_seen = thread::scope(|s| {
            let mut handles = Vec::new();
            for _ in 0..threads {

                let handle = s.spawn(|| {
                    let mut runs = *runs_done.lock().unwrap();
                    let mut local_max = 0;
                    while runs < self.runs_needed {
                        let iterator = model.random_run(initial, bound.clone());
                        for (state, _, _) in iterator {
                            let tokens = state.marking_sum(ctx.get_vars());
                            if tokens > local_max {
                                local_max = tokens;
                            }
                        }
                        {
                            let mut runs_mtx = runs_done.lock().unwrap();
                            *runs_mtx += 1;
                            runs = *runs_mtx;
                        }
                    }
                    local_max
                });

                handles.push(handle);
            }
            let mut threads_max = 0;
            while handles.len() > 0 {
                let local_max = handles.pop().unwrap().join().unwrap();
                threads_max = max(local_max, threads_max);
            }
            threads_max
        });

        let elapsed = now.elapsed().as_secs_f64();
        positive(format!("Estimation complete, max seen : {}", max_seen));
        continue_info(format!("Time elapsed : {}s", elapsed));
        SolverResult::IntResult(max_seen)
    }

}
