use std::{io::Write, sync::{Arc, RwLock}};

use quick_xml::{events::BytesText, ElementWriter, Writer};

use super::{Edge, Label};

pub type NodePos = (f64,f64);

/// Generic trait that should be implemented by all types of nodes to allow automatic rendering
pub trait Node {
    fn get_label(&self) -> Label;
}

pub trait GraphicNode {
    fn design_node<'a, W : Write>(&self, elem : ElementWriter<W>);
    fn write_svg<'a, W : Write>(&self, writer : &'a mut Writer<W>, pos : NodePos);
}

impl<T : Node> GraphicNode for T {
    fn design_node<'a, W : Write>(&self, elem : ElementWriter<W>) {
        let label = self.get_label();
        elem.write_inner_content(|writer| -> Result<(), quick_xml::Error> {
            writer.create_element("circle")
                .with_attributes(vec![("r","50"),("stroke","black"),("stroke-width","5")]);
            writer.create_element("text")
                .with_attributes(vec![("text-anchor","middle"),("dy",".3em")])
                .write_text_content(BytesText::new(label.as_ref()))?;
            Ok(())
        });
    }

    fn write_svg<'a, W : Write>(&self, writer : &'a mut Writer<W>, pos : NodePos) {
        let elem = writer.create_element("g")
            .with_attribute(("id", (/*prefix.clone() +*/ self.get_label()).as_ref()))
            .with_attribute(("transform", format!("translate({} {})", pos.0, pos.1).as_str()));
        self.design_node(elem);
    }
}

impl Node for usize {
    fn get_label(&self) -> Label {
        Label::from(self.to_string())
    }
}

// T is the data type carried by the node, U is the data type carried by the edges
pub struct DataNode<T, U> {
    pub element : T,
    pub out_edges : RwLock<Vec<Arc<Edge<U, Self, Self>>>>, 
    pub in_edges : RwLock<Vec<Arc<Edge<U, Self, Self>>>>,
    pub index : usize,
}

impl<T, U> DataNode<T, U> {

    pub fn downstream_nodes(&self) -> Vec<Arc<Self>> {
        let mut res = Vec::new();
        for edge in self.out_edges.read().unwrap().iter() {
            if !edge.has_target() {
                continue;
            }
            res.push(edge.get_node_to());
        }
        res
    }

    pub fn upstream_nodes(&self) -> Vec<Arc<Self>> {
        let mut res = Vec::new();
        for edge in self.in_edges.read().unwrap().iter() {
            if !edge.has_source() {
                continue;
            }
            res.push(edge.get_node_from());
        }
        res
    }

    pub fn out_degree(&self) -> i32 {
        self.out_edges.read().unwrap().len() as i32
    }

    pub fn in_degree(&self) -> i32 {
        self.in_edges.read().unwrap().len() as i32
    }

    pub fn deg_diff(&self) -> i32 {
        self.out_degree() - self.in_degree()
    }

    pub fn is_sink(&self) -> bool {
        self.out_degree() == 0
    }

    pub fn is_source(&self) -> bool {
        self.in_degree() == 0
    }

    pub fn add_out_edge(&self, edge : &Arc<Edge<U, Self, Self>>) {
        self.out_edges.write().unwrap().push(Arc::clone(&edge));
    }

    pub fn add_in_edge(&self, edge : &Arc<Edge<U, Self, Self>>) {
        self.in_edges.write().unwrap().push(Arc::clone(&edge));
    }

    pub fn clear_edges(&self) {
        self.in_edges.write().unwrap().clear();
        self.out_edges.write().unwrap().clear();
    }

}

impl<T, U> From<T> for DataNode<T, U> {

    fn from(element : T) -> Self {
        DataNode {
            element,
            out_edges : Default::default(),
            in_edges : Default::default(),
            index : 0,
        }
    }

}

impl<T : ToString, U> Node for DataNode<T, U> {

    fn get_label(&self) -> Label {
        Label::from(self.element.to_string())
    }

}

impl<T : Clone, U> Clone for DataNode<T, U> {
    fn clone(&self) -> Self {
        DataNode::from(self.element.clone())
    }
}

impl<T : PartialEq, U> PartialEq for DataNode<T, U> {
    fn eq(&self, other: &Self) -> bool {
        self.element == other.element
    }
}
