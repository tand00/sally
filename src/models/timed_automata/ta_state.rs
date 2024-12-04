use crate::models::{model_clock::ModelClock, time::ClockValue, Label};

#[derive(Debug,Clone)]
pub struct TAState {
    pub name : Label,
    pub invariants : Vec<(ModelClock, ClockValue)>
}

impl TAState {

    pub fn get_name(&self) -> Label {
        self.name.clone()
    }

}
