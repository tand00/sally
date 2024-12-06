use crate::models::{action::Action, ModelObject, ModelState};

pub trait Strategy {

    fn play(&mut self, from : ModelState) -> (ModelState, Vec<Action>);

}

pub struct PlayCombiner<'a> {

    pub model : &'a dyn ModelObject,
    pub strategies : Vec<Box<dyn Strategy>>,

}

impl<'a> PlayCombiner<'a> {

    pub fn combine(&mut self, mut from : ModelState) -> ModelState {
        for strat in self.strategies.iter_mut() {
            let next_state = std::mem::take(&mut from);
            let (next_state, actions) = strat.play(next_state);
            from = next_state;
        }
        from
    }

}

    
