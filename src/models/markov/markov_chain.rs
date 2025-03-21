use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{computation::probability::ProbabilisticChoice, models::{action::Action, lbl, model_context::ModelContext, model_var::{ModelVar, VarType}, time::ClockValue, CompilationResult, Edge, Label, Model, ModelMaker, ModelMeta, ModelState, Node, CONTROLLABLE, STOCHASTIC, UNMAPPED_ID}, verification::{smc::RandomRunIterator, Verifiable, VerificationBound}};

use super::markov_node::MarkovNode;

pub const MarkovActiveNodeVarName : &str = "ActiveNode";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkovChain {
    pub nodes : Vec<MarkovNode>,
    #[serde(skip)]
    pub nodes_dic : HashMap<Label, usize>,
    #[serde(skip)]
    pub id : usize,
    #[serde(skip)]
    pub current_node : ModelVar
}

impl MarkovChain {

    pub fn new(nodes : Vec<MarkovNode>) -> MarkovChain {
        MarkovChain {
            nodes,
            nodes_dic : HashMap::new(),
            id : UNMAPPED_ID,
            current_node : ModelVar::new()
        }
    }

    pub fn get_vars(&self) -> impl Iterator<Item = &ModelVar> {
        self.nodes.iter().map(MarkovNode::get_var)
    }

    pub fn get_current_node(&self, state : &ModelState) -> &MarkovNode {
        let node_index = state.evaluate_var(&self.current_node) as usize;
        &self.nodes[node_index]
    }

    fn build_node_outputs(&self, ctx : &ModelContext, node : &mut MarkovNode) {
        if node.is_choice() {
            node.actions = HashMap::new();
            for (a_label, c) in node.outputs.iter() {
                let action = ctx.get_action(a_label).unwrap_or_else(|| {
                    panic!("Unable to find action ! Maybe node hasn't been compiled");
                });
                let mapped : Vec<(usize, f64)> = c.iter().map(|(l,p)| {
                    (self.nodes_dic[l], *p)
                }).collect();
                let choice = ProbabilisticChoice::new(mapped).normalized();
                node.actions.insert(action, choice);
            }
        } else if node.outputs.len() > 0 {
            for (_, c) in node.outputs.iter() {
                let mapped : Vec<(usize, f64)> = c.iter().map(|(l,p)| {
                    (self.nodes_dic[l], *p)
                }).collect();
                let choice = ProbabilisticChoice::new(mapped).normalized();
                node.actions = HashMap::from([ (Action::Epsilon, choice) ])
            }
        } else {
            node.actions = HashMap::new();
        }

    }

    pub fn get_structure(&self) -> Vec<MarkovNode> {
        self.nodes.clone()
    }

}

impl Model for MarkovChain {

    fn next(&self, mut state : ModelState, action : Action) -> Option<ModelState> {
        let node = self.get_current_node(&state);
        let next_index = node.act(action);
        if next_index == None {
            return None;
        }
        let next_index = next_index.unwrap();
        let next_node = &self.nodes[next_index];
        let actions = next_node.available_actions();
        state.unmark(node.get_var(), 1);
        state.mark(next_node.get_var(), 1);
        state.set_var(&self.current_node, next_index as i32);
        state.deadlocked = actions.len() == 0;
        Some(state)
    }

    fn available_actions(&self, state : &ModelState) -> HashSet<Action> {
        self.get_current_node(state).available_actions()
    }

    fn get_meta() -> ModelMeta {
        ModelMeta {
            name : lbl("MarkovChain"),
            description : String::from("Generic Markov chain, can contain decision nodes"),
            characteristics : CONTROLLABLE | STOCHASTIC
        }
    }

    fn is_timed(&self) -> bool {
        false
    }

    fn is_stochastic(&self) -> bool {
        true
    }

    fn compile(&mut self, context : &mut ModelContext) -> CompilationResult<()> {
        self.id = context.new_model();
        // Not iter_mut in place else we wouldn't be able to borrow self as immut.
        let mut nodes = self.nodes.clone();
        self.nodes_dic = HashMap::new();
        for (i, node) in nodes.iter_mut().enumerate() {
            node.index = i;
            node.compile(context)?;
            self.nodes_dic.insert(node.get_label(), node.index);
        }
        for node in nodes.iter_mut() {
            self.build_node_outputs(context, node);
        }
        self.nodes = nodes;
        self.current_node = context.add_var(lbl(MarkovActiveNodeVarName), VarType::VarU16);
        Ok(())
    }

    fn random_run<'a>(&'a self, initial : &'a ModelState, bound : VerificationBound)
        -> Box<dyn Iterator<Item = (std::rc::Rc<ModelState>, ClockValue, Option<Action>)> + 'a>
    {
        Box::new(RandomRunIterator::generate(self, initial, bound))
    }

    fn get_id(&self) -> usize {
        self.id
    }

    fn nodes_iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Node> + 'a> {
        Box::new(self.nodes.iter().map(|n| n.as_node()))
    }

    fn edges(&self) -> Vec<Edge<String,Label,Label>> {
        todo!()
    }

}

pub struct MarkovChainMaker {
    pub structure : Vec<MarkovNode>
}

impl ModelMaker<MarkovChain> for MarkovChainMaker {

    fn create_maker(model : MarkovChain) -> Self {
        MarkovChainMaker {
            structure : model.get_structure()
        }
    }

    fn make(&self) -> (MarkovChain, ModelContext) {
        let mut chain = MarkovChain::new(self.structure.clone());
        let ctx = chain.singleton().unwrap();
        (chain, ctx)
    }

}
