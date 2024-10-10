use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{computation::probability::ProbabilisticChoice, models::{action::Action, lbl, model_context::ModelContext, model_var::ModelVar, time::ClockValue, CompilationResult, Label, Model, ModelMaker, ModelMeta, ModelState, Node, CONTROLLABLE, STOCHASTIC}, verification::{smc::RandomRunIterator, VerificationBound}};

use std::rc::Rc;

use super::ct_markov_node::CTMarkovNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTMarkovChain {
    pub nodes : Vec<CTMarkovNode>,
    #[serde(skip)]
    pub nodes_dic : HashMap<Label, usize>,
    #[serde(skip)]
    pub id : usize
}

impl Model for CTMarkovChain {
    fn next(&self, mut state : ModelState, action : Action) -> Option<ModelState> {
        todo!()
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<Action> {
        todo!()
    }

    fn get_meta() -> ModelMeta {
        ModelMeta {
            name : lbl("CTMC"),
            description : String::from("Continuous Time Markov chain, can contain decision nodes"),
            characteristics : CONTROLLABLE | STOCHASTIC
        }
    }

    fn is_timed(&self) -> bool {
        true
    }

    fn is_stochastic(&self) -> bool {
        true
    }

    fn compile(&mut self, context : &mut ModelContext) -> CompilationResult<()> {
        todo!()
    }

    fn random_run<'a>(&'a self, initial : &'a ModelState, bound : VerificationBound) 
        -> Box<dyn Iterator<Item = (Rc<ModelState>, ClockValue, Option<Action>)> + 'a> 
    {
        Box::new(RandomRunIterator::generate(self, initial, bound))
    }

    fn get_id(&self) -> usize {
        self.id
    }

}