use std::rc::Weak;

use super::Label;

/// Generic trait that should be implemented by all types of nodes
pub trait Node {
    fn get_label(&self) -> Label;
}

impl Node for usize {
    fn get_label(&self) -> Label {
        Label::from_string(self)
    }
}

pub struct SimpleNode<T> {
    pub element : T,
    pub out_edges : Vec<usize>,
    pub in_edges : Vec<usize>
}

impl<T> SimpleNode<T> {

    pub fn from(element : T) -> Self {
        SimpleNode {
            element,
            out_edges : Vec::new(),
            in_edges : Vec::new()
        }
    }

}

impl<T : ToString + 'static> Node for SimpleNode<T> {
    fn get_label(&self) -> Label {
        Label::from_string(self.element.to_string())
    }
}

impl<T : Clone> Clone for SimpleNode<T> {
    fn clone(&self) -> Self {
        SimpleNode::from(self.element.clone())
    }
}

impl<T : PartialEq> PartialEq for SimpleNode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}
