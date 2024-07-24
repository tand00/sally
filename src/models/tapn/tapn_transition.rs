use std::cmp::min;
use std::collections::HashSet;
use std::fmt;
use std::sync::OnceLock;
use rand::distributions::Distribution;
use serde::{Deserialize, Serialize};

use crate::computation::combinatory::{CartesianProduct, KInVec};
use crate::computation::intervals::{ContinuousSet, Convex};
use crate::models::action::Action;
use crate::models::model_context::ModelContext;
use crate::models::time::{ClockValue, RealTimeInterval, TimeInterval};
use crate::models::{CompilationResult, Label, ModelState, Node};

use super::{tapn_edge::*, TAPNPlaceList, TAPNPlaceListReader, TAPNTokenList, TAPNTokenListReader};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TAPNTransition {
    pub label : Label,
    pub from : Vec<(Label, TAPNEdgeData)>,
    pub to : Vec<(Label, i32)>,
    pub transports : Vec<(Label, Label, TAPNEdgeData)>,
    pub inhibitors : Vec<(Label, TAPNEdgeData)>,
    pub controllable : bool,

    #[serde(skip)]
    pub index : usize,

    #[serde(skip)]
    pub input_edges : OnceLock<Vec<InputEdge>>,
    #[serde(skip)]
    pub output_edges : OnceLock<Vec<OutputEdge>>,
    #[serde(skip)]
    pub inhibitor_edges : OnceLock<Vec<InputEdge>>,
    #[serde(skip)]
    pub transport_edges : OnceLock<Vec<TransportEdge>>,

    #[serde(skip)]
    pub action : Action,
}

impl Node for TAPNTransition {

    fn get_label(&self) -> Label {
        self.label.clone()
    }

}

impl TAPNTransition {

    pub fn new(
        label : Label, 
        from : Vec<(Label, TAPNEdgeData)>, 
        to : Vec<(Label, i32)>,
        inhibitors : Vec<(Label, TAPNEdgeData)>,
        transports : Vec<(Label, Label, TAPNEdgeData)>
    ) -> Self {
        TAPNTransition {
            label,
            from, to, transports, inhibitors,
            controllable : true,
            ..Default::default()
        }
    }

    pub fn get_inputs(&self) -> &Vec<InputEdge> {
        self.input_edges.get().unwrap()
    }

    pub fn get_outputs(&self) -> &Vec<OutputEdge> {
        self.output_edges.get().unwrap()
    }

    pub fn get_transports(&self) -> &Vec<TransportEdge> {
        self.transport_edges.get().unwrap()
    }

    pub fn get_inhibitors(&self) -> &Vec<InputEdge> {
        self.inhibitor_edges.get().unwrap()
    }

    pub fn is_enabled(&self, marking : &ModelState) -> bool {
        for edge in self.get_inputs().iter() {
            if !edge.has_source() {
                panic!("Every transition edge should have a source");
            }
            if marking.tokens(edge.get_node_from().get_var()) < edge.data().weight as i32 {
                return false
            }
        }
        for edge in self.get_transports().iter() {
            if !edge.has_source() {
                panic!("Every transition edge should have a source");
            }
            if marking.tokens(edge.get_node_from().get_var()) < edge.data().weight as i32 {
                return false
            }
        }
        true
    }

    fn has_enough(interval : &TimeInterval, weight : i32, token_list : TAPNTokenListReader) -> bool {
        let mut remaining = weight;
        for token in token_list.tokens() {
            if interval.contains(&token.get_age()) {
                remaining -= *token.count;
                if remaining <= 0 {
                    return true;
                }
            }
        }
        false
    }

    pub fn is_fireable(&self, place_list : &TAPNPlaceListReader) -> bool {
        for inhib in self.get_inhibitors().iter() {
            let place_index = inhib.get_node_from().index;
            let token_list = place_list.place(place_index);
            if Self::has_enough(&inhib.data().interval, inhib.data().weight, token_list) {
                return false;
            }
        }
        for edge in self.get_inputs().iter() {
            let place_index = edge.get_node_from().index;
            let token_list = place_list.place(place_index);
            if !Self::has_enough(&edge.data().interval, edge.data().weight, token_list) {
                return false;
            }
        }
        for edge in self.get_transports().iter() {
            let place_index = edge.get_node_from().index;
            let mut interval = edge.data().interval.clone();
            interval.1 = min(interval.1, edge.get_node_to().invariant);
            let token_list = place_list.place(place_index);
            if !Self::has_enough(&interval, edge.data().weight, token_list) {
                return false;
            }
        }
        true
    }

    fn combinations_for(interval : &TimeInterval, weight : usize, token_list : TAPNTokenListReader) -> Vec<TAPNTokenList> {
        let mut fireable = TAPNTokenList::new();
        for token in token_list.tokens() {
            if interval.contains(&token.get_age()) {
                fireable.append(&mut token.get().flatten());
            }
        }
        if fireable.len() < weight {
            return Vec::new();
        }
        let mut combinations : Vec<TAPNTokenList> = Vec::new();
        for token_set in KInVec::of(weight, &fireable) {
            let mut to_add = TAPNTokenList::new();
            to_add.push(token_set[0].clone());
            for token in token_set.into_iter().skip(1) {
                if token.age == to_add.last().unwrap().age {
                    to_add.last_mut().unwrap().count += 1
                } else {
                    to_add.push(token.clone())
                }
            }
            combinations.push(to_add);
        }
        combinations
    }

    pub fn fireable_tokens(&self, place_list : &TAPNPlaceListReader) -> Vec<TAPNPlaceList> {
        let mut res = Vec::new();
        let mut place_combinations = Vec::new();
        let mut places_index = Vec::new();
        for inhib in self.get_inhibitors().iter() {
            let place_index = inhib.get_node_from().index;
            let token_list = place_list.place(place_index);
            if Self::has_enough(&inhib.data().interval, inhib.data().weight, token_list) {
                return Vec::new();
            }
        }
        for edge in self.get_inputs().iter() {
            let place_index = edge.get_node_from().index;
            places_index.push(place_index);
            let token_list = place_list.place(place_index);
            let combinations = Self::combinations_for(&edge.data().interval, edge.data().weight as usize, token_list);
            if combinations.len() == 0 {
                return Vec::new();
            }
            place_combinations.push(combinations);
        }
        for edge in self.get_transports().iter() {
            let place_index = edge.get_node_from().index;
            places_index.push(place_index);
            let token_list = place_list.place(place_index);
            let mut interval = edge.data().interval.clone();
            interval.1 = min(interval.1, edge.get_node_to().invariant);
            let combinations = Self::combinations_for(&interval, edge.data().weight as usize, token_list);
            if combinations.len() == 0 {
                return Vec::new();
            }
            place_combinations.push(combinations);
        }
        for tuple in CartesianProduct::of(&place_combinations) {
            let mut input_places = TAPNPlaceList::places(place_list.n_places());
            for (i, token_list) in tuple.into_iter().enumerate() {
                let place_index = places_index[i];
                input_places.places[place_index] = token_list.clone();
            }
            res.push(input_places);
        }
        res
    }

    fn arc_dates(interval : &TimeInterval, weight : usize, token_list : TAPNTokenListReader) -> ContinuousSet<ClockValue, RealTimeInterval> {
        let mut dates = ContinuousSet::EmptySet;
        let mut first_index : usize = 0;
        let mut consumed : usize = 0;
        let list_len = token_list.list_len();
        while first_index < list_len {
            let i = first_index;

        }
        dates
    }

    pub fn firing_dates(&self, place_list : &TAPNPlaceListReader) -> ContinuousSet<ClockValue, RealTimeInterval> {
        let mut dates = ContinuousSet::full();
        for edge in self.get_inhibitors().iter() {
            let place_index = edge.get_node_from().index;
            let tokens = place_list.place(place_index);
            let intervals = Self::arc_dates(&edge.data().interval, edge.data().weight as usize, tokens);
            dates = dates.difference(intervals);
            if dates.is_empty() {
                return dates;
            }
        }
        for edge in self.get_inputs().iter() {
            let place_index = edge.get_node_from().index;
            let tokens = place_list.place(place_index);
            let intervals = Self::arc_dates(&edge.data().interval, edge.data().weight as usize, tokens);
            dates = dates.intersection(intervals);
            if dates.is_empty() {
                return dates;
            }
        }
        for edge in self.get_transports().iter() {
            let place_index = edge.get_node_from().index;
            let target_inv = TimeInterval::invariant(edge.get_node_to().invariant);
            let tokens = place_list.place(place_index);
            let interval = edge.data().interval.clone().intersection(target_inv);
            let intervals = Self::arc_dates(&interval, edge.data().weight as usize, tokens);
            dates = dates.intersection(intervals);
            if dates.is_empty() {
                return dates;
            }
        }
        dates
    }

    pub fn clear_edges(&mut self) {
        self.input_edges = OnceLock::new();
        self.output_edges = OnceLock::new();
        self.transport_edges = OnceLock::new();
        self.inhibitor_edges = OnceLock::new();
    }

    pub fn inertia(&self) -> i32 {
        let mut res : i32 = 0;
        for e in self.get_inputs().iter() {
            res -= e.data().weight as i32;
        }
        for e in self.get_outputs().iter() {
            res += e.weight;
        }
        res
    }

    pub fn is_conservative(self) -> bool {
        return self.inertia() == 0
    }

    pub fn set_action(&mut self, action : Action) {
        self.action = action
    }

    pub fn get_action(&self) -> Action {
        self.action.clone()
    }

    pub fn available_actions(&self, place_list : &TAPNPlaceListReader) -> HashSet<Action> {
        let token_sets = self.fireable_tokens(place_list);
        token_sets.into_iter().map(|t| {
            self.action.with_data(t.into())
        }).collect()
    }

    pub fn sample_date(&self) -> ClockValue {
        todo!()
    }

    pub fn has_preset(&self) -> bool {
        self.get_inputs().len() > 0 ||
            self.get_transports().len() > 0 ||
            self.get_inhibitors().len() > 0
    }

    pub fn has_postset(&self) -> bool {
        self.get_outputs().len() > 0 ||
            self.get_transports().len() > 0
    }

    pub fn compile(&mut self, ctx : &mut ModelContext) -> CompilationResult<()> {
        self.set_action(ctx.add_action(self.get_label()));
        Ok(())
    }

}

impl fmt::Display for TAPNTransition {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO : Maybe add from / to in the display text ?
        let from_str : Vec<String> = self.from.iter().map( |lbl| lbl.0.to_string() ).collect();
        let to_str : Vec<String> = self.to.iter().map( |lbl| lbl.0.to_string() ).collect();
        let from_str = from_str.join(",");
        let to_str = to_str.join(",");
        let to_print = format!("Transition_{}_[{}]->[{}]", self.label, from_str, to_str);
        write!(f, "{}", to_print)
    }

}

impl Clone for TAPNTransition {

    fn clone(&self) -> Self {
        TAPNTransition {
            label: self.label.clone(),
            from: self.from.clone(),
            to: self.to.clone(),
            controllable : self.controllable.clone(),
            index : self.index,
            ..Default::default()
        }
    }

}
