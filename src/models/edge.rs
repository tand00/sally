use std::{cell::RefCell, rc::{Rc, Weak}};

use super::{ComponentPtr, Label};

#[derive(Clone)]
pub struct Edge<T, U, V> {
    pub label : Label,
    pub from : Option<Label>,
    pub to : Option<Label>,
    pub weight : T,
    pub ref_from : Option<Weak<RefCell<U>>>,
    pub ref_to : Option<Weak<RefCell<V>>>
}

impl<T, U, V> Edge<T, U, V> {

    pub fn new_weighted(from : Label, to : Label, weight : T) -> Self {
        Edge {
            label: Label::new(),
            from: Some(from), 
            to: Some(to),
            weight,
            ref_from : None,
            ref_to : None
        }
    }

    pub fn node_from(&self) -> Option<ComponentPtr<U>> {
        match &self.ref_from {
            None => None,
            Some(n) => Weak::upgrade(n)
        }
    }

    pub fn node_to(&self) -> Option<ComponentPtr<V>> {
        match &self.ref_to {
            None => None,
            Some(n) => Weak::upgrade(n)
        }
    }

    pub fn ptr_node_from(&self) -> ComponentPtr<U> {
        self.node_from().unwrap()
    }

    pub fn ptr_node_to(&self) -> ComponentPtr<V> {
        self.node_to().unwrap()
    }

    pub fn set_node_from(&mut self, node : &ComponentPtr<U>) {
        self.ref_from = Some(Rc::downgrade(node))
    }

    pub fn set_node_to(&mut self, node : &ComponentPtr<V>) {
        self.ref_to = Some(Rc::downgrade(node))
    }

    pub fn has_source(&self) -> bool {
        match &self.ref_from {
            Some(r) => r.strong_count() > 0,
            None => false
        }
    }

    pub fn has_target(&self) -> bool {
        match &self.ref_to {
            Some(r) => r.strong_count() > 0,
            None => false
        }
    }

}

impl<U, V> Edge<i32, U, V> {
    pub fn new(from : Label, to : Label) -> Self {
        Edge::new_weighted(from, to, 1)
    }
}