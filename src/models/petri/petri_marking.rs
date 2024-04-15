type Tokens = i32;

#[derive(Clone, Hash)]
pub struct PetriMarking {
    tokens: Vec<Tokens>
}

impl PetriMarking {

    pub fn is_marked(&self, place : usize) -> bool {
        self.tokens[place] > 0
    }

    pub fn tokens(&self, place: usize) -> Tokens {
        self.tokens[place]
    }

    pub fn marked_places(&self) -> Vec<usize> {
        let mut res : Vec<usize> = Vec::new();
        for (pos, value) in self.tokens.iter().enumerate() {
            if *value > 0 {
                res.push(pos);
            }
        }
        res
    }

    pub fn unmark(&mut self, place : usize, tokens : Tokens) {
        self.tokens[place] -= tokens;
    }

    pub fn mark(&mut self, place : usize, tokens : Tokens) {
        self.tokens[place] += tokens;
    }

}