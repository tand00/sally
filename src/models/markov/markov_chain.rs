use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::models::{action::Action, lbl, model_context::ModelContext, model_var::ModelVar, CompilationResult, Label, Model, ModelMaker, ModelMeta, ModelState, Node, CONTROLLABLE, STOCHASTIC};

use super::{markov_node::MarkovNode, ProbabilisticChoice};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkovChain {
    pub nodes : Vec<MarkovNode>,
    #[serde(skip)]
    pub nodes_dic : HashMap<Label, usize>,
    #[serde(skip)]
    pub id : usize
}

impl MarkovChain {

    pub fn new(nodes : Vec<MarkovNode>) -> MarkovChain {
        MarkovChain {
            nodes,
            nodes_dic : HashMap::new(),
            id : usize::MAX
        }
    }

    pub fn get_vars(&self) -> impl Iterator<Item = &ModelVar> {
        self.nodes.iter().map(|n| n.get_var() )
    }

    pub fn get_current_node(&self, state : &ModelState) -> &MarkovNode {
        let node_index = state.argmax(self.get_vars());
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

    fn next(&self,mut state : ModelState, action : Action) -> Option<(ModelState, HashSet<Action>)> {
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
        state.deadlocked = actions.len() == 0;
        Some((state, actions))
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
        Ok(())
    }

    fn get_id(&self) -> usize {
        self.id
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
        let ctx = chain.singleton();
        (chain, ctx)
    }

}