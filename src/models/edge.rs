use super::Label;

#[derive(Clone)]
pub struct Edge {
    pub label : Label,
    pub from : Option<Label>,
    pub to : Option<Label>,
    pub weight : i16,
    r_from_: Option<usize>,
    r_to_: Option<usize>
}

impl Edge {

    pub fn new(from : Label, to : Label) -> Self {
        Edge {
            label: Label::new(),
            from: Some(from), 
            to: Some(to),
            weight: 1,
            r_from_: None,
            r_to_: None
        }
    }

    pub fn new_weighted(from : Label, to : Label, weight : i16) -> Edge {
        Edge {
            label: Label::new(),
            from: Some(from), 
            to: Some(to),
            weight,
            r_from_: None,
            r_to_: None
        }
    }

    pub fn node_from(&self) -> usize {
        self.r_from_.clone().unwrap()
    }
    pub fn node_to(&self) -> usize {
        self.r_to_.clone().unwrap()
    }
    pub fn set_node_from(&mut self, node : usize) {
        self.r_from_ = Some(node)
    }
    pub fn set_node_to(&mut self, node : usize) {
        self.r_to_ = Some(node)
    }
    pub fn has_source(&self) -> bool {
        match self.r_from_ {
            Some(_) => true,
            None => false
        }
    }
    pub fn has_target(&self) -> bool {
        match self.r_to_ {
            Some(_) => true,
            None => false
        }
    }
}