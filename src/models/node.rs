use super::Label;

/// Generic trait that should be implemented by all types of states
pub trait Node {
    fn get_label(&self) -> Label;
    fn clone_box(&self) -> Box<dyn Node>;
}
impl Clone for Box<dyn Node> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}