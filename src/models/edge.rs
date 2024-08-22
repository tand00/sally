use std::sync::{Arc, Weak};

use serde::{Deserialize, Serialize};

use super::Label;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge<T, U, V> {
    pub from : Option<Label>,
    pub to : Option<Label>,
    pub weight : T,
    #[serde(skip)]
    pub ref_from : Option<Weak<U>>,
    #[serde(skip)]
    pub ref_to : Option<Weak<V>>,
}

impl<T, U, V> Edge<T, U, V> {

    pub fn orphan(data : T) -> Self {
        Edge {
            from : None,
            to : None,
            weight : data,
            ref_from : None,
            ref_to : None
        }
    }

    pub fn new_weighted(from : Label, to : Label, weight : T) -> Self {
        Edge {
            from: Some(from), 
            to: Some(to),
            weight,
            ref_from : None,
            ref_to : None,
        }
    }

    pub fn data_edge(from : &Arc<U>, to : &Arc<V>, weight : T) -> Self {
        Edge {
            from: None, 
            to: None,
            weight,
            ref_from : Some(Arc::downgrade(from)),
            ref_to : Some(Arc::downgrade(to)),
        }
    }

    pub fn node_from(&self) -> Option<Arc<U>> {
        match &self.ref_from {
            None => None,
            Some(n) => Weak::upgrade(n)
        }
    }

    pub fn node_to(&self) -> Option<Arc<V>> {
        match &self.ref_to {
            None => None,
            Some(n) => Weak::upgrade(n)
        }
    }

    pub fn data(&self) -> &T {
        &self.weight
    }

    pub fn get_node_from(&self) -> Arc<U> {
        self.node_from().unwrap()
    }

    pub fn get_node_to(&self) -> Arc<V> {
        self.node_to().unwrap()
    }

    pub fn set_node_from(&mut self, node : &Arc<U>) {
        self.ref_from = Some(Arc::downgrade(node))
    }

    pub fn set_node_to(&mut self, node : &Arc<V>) {
        self.ref_to = Some(Arc::downgrade(node))
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