use super::{Label, Model, State, Transition};
use super::time::TimeInterval;
use std::fmt;
use std::collections::HashMap;

#[derive(Debug,Clone)]
struct ModelCompilationError(String);
impl fmt::Display for ModelCompilationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Model Compilation Error : {}", &self.0)
    }
}

pub struct ClassGraphTransition {
    pub from : Vec<Label>,
    pub to : Vec<Label>,
    pub interval : TimeInterval,
}

pub struct ClassGraphClass {
    pub sub_states : Vec<Label>,
    pub domain : HashMap<Label, TimeInterval>,
}

pub struct ClassGraph {
    pub classes : Vec<ClassGraphClass>,
    pub transitions : Vec<ClassGraphTransition>,
    pub initial_states : Vec<Label>,
}
impl ClassGraph {

    pub fn class_for_states(&self, states : Vec<Label>) -> Option<&ClassGraphClass> {
        for graphClass in self.classes.iter() {
            if graphClass.sub_states == states {
                return Some(graphClass)
            }
        }
        None
    }

    pub fn find_initial_class(&self) -> Option<&ClassGraphClass> {
        self.class_for_states(self.initial_states)
    }

    pub fn outgoing_transitions(&self, state : &ClassGraphClass) -> Vec<&ClassGraphTransition> {
        let mut out_transi : Vec<&ClassGraphTransition> = Vec::new();
        for transi in self.transitions.iter() {
            if transi.from == state.sub_states {
                out_transi.push(transi);
            }
        }
        out_transi
    }

    pub fn compile(&self) -> Result<CompiledClassGraph, ModelCompilationError> {
        let mut compiled = CompiledClassGraph {
            initial: None, links: Vec::new(), nodes: Vec::new()
        };
        let mut initial = self.find_initial_class();
        if initial.is_none() { 
            return Err(ModelCompilationError(String::from("Could not find intial state"))) 
        }
        compiled.nodes.push(ClassGraphNode { state: initial.unwrap(), links: HashMap::new() });
        compiled.initial = Some(&compiled.nodes[0]);
        let mut to_explore : Vec<Vec<Label>>;
        let mut explored = vec![compiled.nodes[0].state.sub_states];

        Ok(compiled)
    }

}

pub struct ClassGraphLink<'a> {
    pub transition : &'a ClassGraphTransition,
    pub from : Box<&'a ClassGraphNode<'a>>,
    pub to : Box<&'a ClassGraphNode<'a>>,
}

pub struct ClassGraphNode<'a> {
    pub state: &'a ClassGraphClass,
    pub links: HashMap<Label, &'a ClassGraphLink<'a>>,
}

pub struct CompiledClassGraph<'a> {
    pub initial : Option<&'a ClassGraphNode<'a>>,
    pub links : Vec<ClassGraphLink<'a>>,
    pub nodes : Vec<ClassGraphNode<'a>>,
}