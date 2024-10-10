#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Word(Vec<usize>);

impl Word {

    pub fn new() -> Word {
        Word(Vec::new())
    }

    pub fn add_letter(&mut self, letter : usize) {
        self.0.push(letter);
    }

}