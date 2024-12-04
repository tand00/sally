use crate::models::{expressions::Condition, model_context::ModelContext, CompilationError, CompilationResult, Label};

#[derive(Debug,Clone)]
pub struct TAState {
    pub name : Label,
    pub invariants : Condition
}

impl TAState {

    pub fn get_name(&self) -> Label {
        self.name.clone()
    }

    pub fn compile(&mut self, ctx : &mut ModelContext) -> CompilationResult<()> {
        let Ok(cond) = self.invariants.apply_to(ctx) else {
            return Err(CompilationError)
        };
        self.invariants = cond;
        Ok(())
    }

}
