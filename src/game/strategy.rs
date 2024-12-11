use std::collections::HashSet;

use crate::models::{action::Action, ModelObject, ModelState};

use super::arena::Player;

pub trait Strategy {
    fn play(&mut self, from : ModelState, actions : HashSet<Action>) -> (ModelState, Vec<Action>);
}

pub trait PlayCombiner {
    fn combine(&mut self, players : &mut Vec<Player>, from : ModelState) -> ModelState;
}

pub struct SequentialPlayCombiner<'a> {
    pub model : &'a dyn ModelObject,
}

pub struct FinalChoosePlayCombiner<'a> {
    pub model : &'a dyn ModelObject,
}

impl<'a> PlayCombiner for SequentialPlayCombiner<'a> {
    fn combine(&mut self, players : &mut Vec<Player>, mut from : ModelState) -> ModelState {
        for player in players.iter_mut() {
            let next_state = std::mem::take(&mut from);
            let avail = self.model.available_actions(&next_state);
            let (mut next_state, actions) = player.strategy.play(next_state, avail);
            if let Some(action) = actions.iter().next() {
                next_state = self.model.next(next_state, action.clone()).unwrap();
            }
            from = next_state;
        }
        from
    }
}
