use std::{collections::HashSet, rc::Rc};

use num_traits::Zero;

use crate::{computation::intervals::ContinuousSet, models::{action::Action, run::RunStatus, time::{ClockValue, RealTimeBound, RealTimeInterval}, Model, ModelState}, verification::VerificationBound};

use super::{TAPNPlaceList, TAPNPlaceListReader, TAPN};

pub struct TAPNRunGenerator<'a> {
    pub tapn : &'a TAPN,
    pub initial_state : &'a ModelState,
    pub bound : VerificationBound,
    pub intervals : Vec<ContinuousSet<ClockValue, RealTimeInterval>>,
    pub firing_dates : Vec<ClockValue>,
    pub started : bool,
    pub run_status : RunStatus,
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
            }
        };
        generator.init_intervals();
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
        self.init_intervals();
    }

    pub fn init_intervals(&mut self) {
        let state = &self.run_status.current_state;
        let avail_delay = self.tapn.available_delay(&self.run_status.current_state);
        let avail_delay : ContinuousSet<ClockValue, RealTimeInterval> = RealTimeInterval::invariant(avail_delay).into();
        let place_list = TAPNPlaceListReader::from(state.storage(&self.tapn.tokens_storage));
        for i in 0..self.intervals.len() {
            let transition = &self.tapn.transitions[i];
            let transi_dates = transition.firing_dates(&place_list);
            self.intervals[i] = transi_dates.intersection(avail_delay.clone());
            if self.intervals[i].contains(&ClockValue::zero()) {
                self.firing_dates[i] = transition.sample_date();
            }
        }
    }

    pub fn refresh_intervals(&mut self, modified_places : HashSet<usize>) {
        let state = &self.run_status.current_state;
        let avail_delay = self.tapn.available_delay(&self.run_status.current_state);
        let avail_delay : ContinuousSet<ClockValue, RealTimeInterval> = RealTimeInterval::invariant(avail_delay).into();
        let place_list = TAPNPlaceListReader::from(state.storage(&self.tapn.tokens_storage));
        let mut transition_seen = vec![false ; self.intervals.len()];
        for place_index in modified_places.iter() {
            let place = &self.tapn.places[*place_index];
            for transition in place.get_downstream_transitions().iter() {
                if transition_seen[transition.index] {
                    continue;
                }
                transition_seen[transition.index] = true;
                let dates = transition.firing_dates(&place_list).intersection(avail_delay.clone());
                let enabled = dates.contains(&ClockValue::zero());
                let newly_enabled = enabled && self.firing_dates[transition.index].is_disabled();
                self.intervals[transition.index] = dates;
                if !enabled {
                    self.firing_dates[transition.index] = ClockValue::disabled();
                } else if newly_enabled {
                    self.firing_dates[transition.index] = transition.sample_date();
                }
            }
        }
    }

    pub fn get_winner_and_delay(&self) -> (Option<usize>, ClockValue) {
        let mut delay = ClockValue::infinity(); 
        let mut candidates : Vec<usize> = Vec::new();
        for i in 0..self.intervals.len() {
            let dates = &self.intervals[i];
            if dates.is_empty() {
                continue;
            }
            let first = dates.convexs().next().unwrap();
            if first.1 > RealTimeBound::zero() && first.1.lower_than(&delay) {
                
            }
        }

        (None, delay)
    }

    pub fn random_token_set(&self, transition : usize, place_list : TAPNPlaceListReader) -> TAPNPlaceList {
        let transition = &self.tapn.transitions[transition];
        todo!()
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
        if let Some(winner) = winner {
            let place_list = TAPNPlaceListReader::from(next_state.storage(&self.tapn.tokens_storage));
            let in_tokens = self.random_token_set(winner, place_list);
            let (next_state, modified) = self.tapn.fire(next_state, winner, in_tokens);
            self.refresh_intervals(modified);
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