use crate::models::ModelState;

pub trait Strategy {

    type Input;
    type Output;

    fn play(&mut self, from : &Self::Input) -> Self::Output;

}

pub trait PlayCombiner {

    type Input;
    type Output;

    fn combine(
        &mut self,
        strategies : &mut Vec<Box<dyn Strategy<Input = Self::Input, Output = Self::Output>>>,
        from : &Self::Input
    ) -> Self::Input;

}
