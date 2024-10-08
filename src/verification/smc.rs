mod random_run_generator;
mod probability_estimation;
mod probability_float_comparison;
mod smc_max_seen;

use std::{sync::{mpsc, Mutex}, thread, time::Instant};

pub use random_run_generator::RandomRunIterator;
pub use probability_estimation::ProbabilityEstimation;
pub use probability_float_comparison::ProbabilityFloatComparison;
pub use smc_max_seen::SMCMaxSeen;

use crate::{models::{Model, ModelObject, ModelState}, solution::SolverResult};

use super::{query::Query, Verifiable, VerificationStatus};

use crate::log::*;

pub trait SMCQueryVerification {

    // Required implementations
    fn must_do_another_run(&self) -> bool;
    fn handle_run_result(&mut self, result : VerificationStatus);
    fn get_result(&self) -> SolverResult;

    // Optional implementations
    fn prepare(&self) { }
    fn finish(&self) { }

    // Default implementations
    fn verify(&mut self, model : &dyn ModelObject, initial_state : &ModelState, query : &Query) -> SolverResult {
        info("SMC verification");
        self.prepare();
        pending("Starting...");
        let now = Instant::now();
        let mut query = query.clone();
        while self.must_do_another_run() {
            let result = Self::execute_run(model, initial_state, &mut query);
            self.handle_run_result(result);
        }
        self.finish();
        let elapsed = now.elapsed().as_secs_f64();
        positive("Verification finished");
        continue_info(format!("Time elapsed : {}s", elapsed));
        self.get_result()
    }

    fn execute_run(model : &dyn ModelObject, initial_state : &ModelState, query : &mut Query) -> VerificationStatus {
        let run_gen = model.random_run(initial_state, query.run_bound.clone());
        for (state, _, _) in run_gen {
            query.verify_state(state.as_verifiable());
            if query.is_run_decided() {
                break;
            }
        }
        query.end_run();
        let result = query.run_status;
        query.reset_run();
        result
    }

    fn parallel_verify(&mut self, model : &dyn ModelObject, initial_state : &ModelState, query : &Query) -> SolverResult {
        info("SMC verification");
        let threads = thread::available_parallelism().unwrap().get();
        continue_info(format!("Parallel mode [Threads : {}]", threads));
        self.prepare();
        pending("Starting...");
        let now = Instant::now();

        let (tx,rx) = mpsc::channel::<VerificationStatus>();
        let must_continue = Mutex::new(true);

        thread::scope(|s| {
            let mut handles = Vec::new();
            for _ in 0..threads {

                let handle = s.spawn(|| {
                    let mut thread_query = query.clone();
                    let mut must_do_another = *must_continue.lock().unwrap();
                    while must_do_another {
                        let result = Self::execute_run(model, initial_state, &mut thread_query);
                        tx.send(result).expect("Unable to send result !");
                        must_do_another = *must_continue.lock().unwrap();
                    }
                });

                handles.push(handle);
            }

            for received in rx {
                self.handle_run_result(received);
                if !self.must_do_another_run() {
                    {
                        let mut threads_guard = must_continue.lock().unwrap();
                        *threads_guard = false;
                    }
                    for handle in handles {
                        handle.join().unwrap();
                    }
                    break;
                }
            }
        });

        self.finish();
        let elapsed = now.elapsed().as_secs_f64();
        positive("Verification finished");
        continue_info(format!("Time elapsed : {}s", elapsed));
        self.get_result()
    }

}
