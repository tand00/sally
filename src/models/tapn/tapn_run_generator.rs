use std::rc::Rc;

use num_traits::Zero;

use crate::{computation::{convex::{ContinuousSet, Delta, ToPositive}, probability::ProbabilisticChoice}, models::{action::Action, run::RunStatus, time::{ClockValue, RealTimeInterval}, Model, ModelState}, verification::VerificationBound};

use super::{tapn_transition::{FiringMode, TAPNTransition}, TAPNPlaceList, TAPNPlaceListReader, TAPN};

use rand::{rngs::ThreadRng, seq::SliceRandom, thread_rng, Rng};

pub struct TAPNRunGenerator<'a> {
    pub tapn : &'a TAPN,
    pub initial_state : &'a ModelState,
    pub bound : VerificationBound,
    pub intervals : Vec<ContinuousSet<ClockValue, RealTimeInterval>>,
    pub firing_dates : Vec<ClockValue>,
    pub started : bool,
    pub run_status : RunStatus,
    pub rng : ThreadRng,
}

impl<'a> TAPNRunGenerator<'a> {

    pub fn generate(tapn : &'a TAPN, initial_state : &'a ModelState, bound : VerificationBound) -> Self {
        let mut generator = TAPNRunGenerator {
            tapn,
            initial_state,
            bound,
            intervals : vec![ContinuousSet::EmptySet ; tapn.transitions.len()],
            firing_dates : vec![ClockValue::disabled() ; tapn.transitions.len()],
            started : false,
            run_status : RunStatus {
                current_state : Rc::new(initial_state.clone()),
                steps : 0,
                time : ClockValue::zero(),
                maximal : false
            },
            rng : thread_rng()
        };
        generator.refresh_intervals();
        generator
    }

    pub fn reset(&mut self) {
        self.run_status = RunStatus {
            current_state : Rc::new(self.initial_state.clone()),
            steps : 0,
            time : ClockValue::zero(),
            maximal : false
        };
        self.started = false;
        self.refresh_intervals();
    }

    pub fn refresh_intervals(&mut self) {
        let state = &self.run_status.current_state;
        let avail_delay = self.tapn.available_delay(&self.run_status.current_state);
        let avail_delay : ContinuousSet<ClockValue, RealTimeInterval> = RealTimeInterval::invariant(avail_delay).into();
        let place_list = TAPNPlaceListReader::from(state.storage(&self.tapn.tokens_storage));
        for transition in self.tapn.transitions.iter() {
            let dates = transition.firing_dates(&place_list).intersection(avail_delay.clone());
            let enabled = dates.contains(&ClockValue::zero());
            let newly_enabled = enabled && self.firing_dates[transition.index].is_disabled();
            let reached_upper = enabled && dates.convexs().next().unwrap().1.clock_value().is_zero();
            self.intervals[transition.index] = dates;
            if !enabled {
                self.firing_dates[transition.index] = ClockValue::disabled();
            } else if newly_enabled {
                self.firing_dates[transition.index] = transition.sample_date(&mut self.rng);
            }
            if reached_upper && self.firing_dates[transition.index] > ClockValue::zero() {
                self.firing_dates[transition.index] = ClockValue::disabled();
            }
        }
    }

    pub fn get_winner_and_delay(&mut self) -> (Option<usize>, ClockValue) {
        let mut delay = ClockValue::infinity();
        let mut candidates : Vec<(usize, f64)> = Vec::new();
        let mut infinite_weights : Vec<usize> = Vec::new();
        let mut null_weights : Vec<usize> = Vec::new();
        for t in self.tapn.transitions.iter() {
            let i = t.index;
            let dates = &self.intervals[i];
            let firing = &self.firing_dates[i];
            if dates.is_empty() {
                continue;
            }
            let mut date = ClockValue::infinity();
            for convex in dates.convexs() {
                let a = convex.0.clock_value();
                let b = convex.1.clock_value();
                if a > ClockValue::zero() {
                    date = a;
                    break;
                }
                if b > ClockValue::zero() {
                    date = b;
                    break;
                }
            }
            if firing.is_enabled() && *firing < date {
                date = *firing;
            }
            if date < delay {
                delay = date;
                candidates.clear();
            }
            if *firing == delay {
                if t.weight.is_infinite() {
                    infinite_weights.push(i);
                } else if t.weight.is_zero() {
                    null_weights.push(i);
                } else {
                    candidates.push((i, t.weight));
                }
            }
        }
        let winner = match (candidates.is_empty(), infinite_weights.is_empty(), null_weights.is_empty()) {
            (true, true, true) => None,
            (_, false, _) => infinite_weights.choose(&mut self.rng).map(|i| *i),
            (true, true, false) => null_weights.choose(&mut self.rng).map(|i| *i),
            (false, true, _) => {
                let choice = ProbabilisticChoice::new(candidates);
                Some(*choice.sample(&mut self.rng))
            }
        };
        (winner, delay)
    }

    pub fn select_token_set(&mut self, transition : usize, place_list : TAPNPlaceListReader) -> TAPNPlaceList {
        let transition = &self.tapn.transitions[transition];
        match transition.firing_mode {
            FiringMode::Oldest => self.oldest_token_set(transition, place_list),
            FiringMode::Youngest => self.youngest_token_set(transition, place_list),
            FiringMode::Random => self.random_token_set(transition, place_list),
        }
    }

    pub fn oldest_token_set(&self, transition : &TAPNTransition, place_list : TAPNPlaceListReader) -> TAPNPlaceList {
        let mut token_sets = transition.fireable_tokens(&place_list);
        token_sets.pop().unwrap()
    }

    pub fn youngest_token_set(&self, transition : &TAPNTransition, place_list : TAPNPlaceListReader) -> TAPNPlaceList {
        let mut token_sets = transition.fireable_tokens(&place_list);
        token_sets.swap_remove(0)
    }

    pub fn random_token_set(&mut self, transition : &TAPNTransition, place_list : TAPNPlaceListReader) -> TAPNPlaceList {
        let mut token_sets = transition.fireable_tokens(&place_list);
        let random_index = self.rng.gen_range(0..token_sets.len());
        token_sets.swap_remove(random_index)
    }

    pub fn disable_transitions(&mut self, places : &TAPNPlaceListReader) {
        for transition in self.tapn.transitions.iter() {
            if !transition.is_fireable(places) {
                self.firing_dates[transition.index] = ClockValue::disabled();
            }
        }
    }

    pub fn time_forward(&mut self, delay : ClockValue) {
        for i in 0..self.firing_dates.len() {
            let date = &mut self.firing_dates[i];
            if date.is_enabled() {
                *date -= delay;
                if *date < ClockValue::zero() {
                    *date = ClockValue::zero();
                }
            }
            let interval = &mut self.intervals[i];
            interval.delta(-delay);
            interval.into_positive();
        }
    }

}

impl<'a> Iterator for TAPNRunGenerator<'a> {

    type Item = (Rc<ModelState>, ClockValue, Option<Action>);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.started {
            self.started = true;
            return Some((self.run_status.current_state.clone(), ClockValue::zero(), None))
        }
        let next_state = ModelState::clone(&self.run_status.current_state);
        let (winner, delay) = self.get_winner_and_delay();
        let Some(next_state) = self.tapn.delay(next_state, delay) else {
            return None;
        };
        self.run_status.time += delay;

        self.time_forward(delay);

        if let Some(winner) = winner {
            let place_list = TAPNPlaceListReader::from(next_state.storage(&self.tapn.tokens_storage));
            let in_tokens = self.select_token_set(winner, place_list);
            let next_state = self.tapn.fire(next_state, winner, in_tokens);
            self.firing_dates[winner] = ClockValue::disabled();
            // self.disable_transitions(&TAPNPlaceListReader::from(&intermed));
            self.refresh_intervals();
            self.run_status.steps += 1;
            self.run_status.current_state = Rc::new(next_state);
            let action = self.tapn.transitions[winner].get_action();
            return Some((Rc::clone(&self.run_status.current_state), delay, Some(action)));
        } else {
            self.run_status.current_state = Rc::new(next_state);
            return Some((Rc::clone(&self.run_status.current_state), delay, None));
        }
    }

}
