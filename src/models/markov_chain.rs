use std::{collections::HashMap, fmt::Display};

use super::{digraph::Digraph, ComponentPtr, Label, Model};

#[derive(Debug, Clone, PartialEq)]
pub enum MarkovNode {
    ProbabilisticNode(usize),
    ChoiceNode(usize),
    ActionNode(usize)
}
impl Display for MarkovNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProbabilisticNode(l) | Self::ChoiceNode(l) 
                => write!(f, "MarkovNode({})", l),
            Self::ActionNode(l) => write!(f, "Action({})", l)
        }
    }
}
pub struct MarkovChain {
    pub graph : Digraph<MarkovNode, f64>,
    pub var_dic : HashMap<Label, usize>,
    pub actions_dic : HashMap<Label, usize>,
    pub var_nodes : Vec<ComponentPtr<MarkovNode>>,
}

impl Model for MarkovChain {

    fn get_meta() -> super::ModelMeta where Self : Sized {
        
    }

    fn available_actions(&self, state : &super::ModelState) -> std::collections::HashSet<usize> {
        let (var, _) = state.discrete.argmax();
        let node = Rc::clone(self.var_nodes[var]);
        
    }

    fn n_vars(&self) -> usize {
        self.var_dic.len()
    }

    fn next(&self, state : super::ModelState, action : usize) -> (Option<super::ModelState>, std::collections::HashSet<usize>) {
        
    }

    fn random_next(&self, state : super::ModelState) -> (Option<super::ModelState>, super::time::ClockValue, Option<usize>) {
        
    }

    fn map_label_to_var(&self, var : &Label) -> Option<usize> {
        if self.var_dic.contains_key(var) {
            Some(self.var_dic[var])
        } else {
            None
        }
    }

}