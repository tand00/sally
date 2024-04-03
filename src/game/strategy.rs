pub trait Strategy {
    type Input;
    type Output;

    fn play(&mut self, from : Self::Input) -> Self::Output;

}