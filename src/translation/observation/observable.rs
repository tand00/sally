use std::cmp::max;

use crate::{computation::virtual_memory::EvaluationType, models::{action::Action, class_graph::StateClass, ModelState}, verification::Verifiable};

use super::function::{ObservationContext, ObservationFunction, VarPolicy};

pub trait Observable {
    type Observed;

    fn observe(&self, ctx : &ObservationContext, fun : &ObservationFunction) -> Self::Observed;
}

impl Observable for ModelState {
    type Observed = Self;

    fn observe(&self, ctx : &ObservationContext, fun : &ObservationFunction) -> Self::Observed {
        let mut observed = ctx.observed.make_empty_state();
        let var_junction = match fun.var_policy {
            VarPolicy::SumVars => |x,y| x + y,
            VarPolicy::MaxVar => |x,y| max(x, y),
            VarPolicy::UnitVar => |x,y| if x > 0 || y > 0 { 1 } else { 0 },
        };
        for (x,o) in ctx.links.vars.iter() {
            let value = var_junction(self.evaluate_var(x), observed.evaluate_var(o));
            observed.set_marking(o, value);
        }
        for (x,o) in ctx.links.clocks.iter() {
            if self.is_enabled(x) {
                observed.set_clock(o, self.get_clock_value(x));
                break;
            }
        }
        observed.storages = self.storages.clone();
        observed.deadlocked = self.deadlocked;
        observed
    }

}

impl Observable for Action {
    type Observed = Self;

    fn observe(&self, ctx : &ObservationContext, _fun : &ObservationFunction) -> Self::Observed {
        let base = self.base();
        if !ctx.links.actions.contains_key(&base) {
            return Action::Epsilon;
        }
        let result = ctx.links.actions[&base].clone();
        match self {
            Action::Epsilon => Action::Epsilon,
            Action::Base(_) => result,
            Action::Sync(_, a, b) => result.sync(Action::clone(a), Action::clone(b)),
            Action::WithData(_, d) => result.with_data(d.clone())
        }
    }

}

impl Observable for StateClass {
    type Observed = Self;

    fn observe(&self, ctx : &ObservationContext, fun : &ObservationFunction) -> Self::Observed {
        todo!()
    }


}
