

use std::fmt::Display;

use crate::models::{model_storage::ModelStorage, time::ClockValue};

// ----- Token -----

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct TAPNToken {
    pub count : i32,
    pub age : ClockValue
}

impl TAPNToken {

    pub fn flatten(self) -> Vec<TAPNToken> {
        (0..self.count).map(|_| {
            TAPNToken { count : 1, age : self.age }
        }).collect()
    }

}

impl Display for TAPNToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tokens({}x{})", self.age, self.count)
    }
}

pub struct TAPNTokenReader<'a> {
    pub count : &'a i32,
    pub age : &'a f64
}
pub struct TAPNTokenWriter<'a> {
    pub count : &'a mut i32,
    pub age : &'a mut f64
}

impl<'a> TAPNTokenReader<'a> {

    pub fn get(&self) -> TAPNToken {
        TAPNToken {
            count : *self.count,
            age : ClockValue::from(*self.age)
        }
    }

    pub fn get_age(&self) -> ClockValue {
        ClockValue::from(*self.age)
    }

}
impl<'a> TAPNTokenWriter<'a> {

    pub fn get(&self) -> TAPNToken {
        TAPNToken {
            count : *self.count,
            age : ClockValue::from(*self.age)
        }
    }

    pub fn get_age(&self) -> ClockValue {
        ClockValue::from(*self.age)
    }

}

impl From<ModelStorage> for TAPNToken {
    fn from(value: ModelStorage) -> Self {
        let (count, age) = value.tuple();
        let count = count.int();
        let age = ClockValue::from(age.float());
        TAPNToken { count, age }
    }
}
impl<'a> From<&'a ModelStorage> for TAPNTokenReader<'a> {
    fn from(value: &'a ModelStorage) -> Self {
        let (count, age) = value.ref_tuple();
        let count = count.ref_int();
        let age = age.ref_float();
        TAPNTokenReader { count, age }
    }
}
impl<'a> From<&'a mut ModelStorage> for TAPNTokenWriter<'a> {
    fn from(value: &'a mut ModelStorage) -> Self {
        let (count, age) = value.mut_tuple();
        let count = count.mut_int();
        let age = age.mut_float();
        TAPNTokenWriter { count, age }
    }
}
impl From<TAPNToken> for ModelStorage {
    fn from(value: TAPNToken) -> Self {
        let count = ModelStorage::from(value.count);
        let age = ModelStorage::from(value.age.float());
        (count, age).into()
    }
}

// ----- TokenList -----

pub type TAPNTokenList = Vec<TAPNToken>;
impl From<ModelStorage> for TAPNTokenList {
    fn from(value : ModelStorage) -> Self {
        let vec = value.vec();
        vec.into_iter().map(|x| TAPNToken::from(x) ).collect()
    }
}
impl From<TAPNTokenList> for ModelStorage {
    fn from(value : TAPNTokenList) -> Self {
        let value : Vec<ModelStorage> = value.into_iter().map(|t| ModelStorage::from(t) ).collect();
        value.into()
    }
}

pub struct TAPNTokenListReader<'a> {
    tokens: &'a Vec<ModelStorage> //Vec<TAPNTokenWriter<'a>>
}
pub struct TAPNTokenListWriter<'a> {
    tokens: &'a mut Vec<ModelStorage> //Vec<TAPNTokenWriter<'a>>
}

impl<'a> TAPNTokenListReader<'a> {

    pub fn tokens(&self) -> impl Iterator<Item = TAPNTokenReader> {
        self.tokens.iter().map(|t| TAPNTokenReader::from(t) )
    }
    
    pub fn n_tokens(&self) -> i32 {
        self.tokens().map(|t| *t.count ).sum()
    }

    pub fn get(&self) -> TAPNTokenList {
        self.tokens().map(|t| t.get()).collect()
    }

    pub fn max_age(&self) -> ClockValue {
        if self.tokens.len() == 0 {
            return ClockValue::neg_infinity();
        }
        let last_age = *self.tokens.last().unwrap().ref_tuple().1.ref_float();
        ClockValue::from(last_age)
    }

    pub fn list_len(&self) -> usize {
        self.tokens.len()
    }

}
impl<'a> TAPNTokenListWriter<'a> {

    pub fn tokens(&mut self) -> impl Iterator<Item = TAPNTokenWriter> {
        self.tokens.iter_mut().map(|x| TAPNTokenWriter::from(x) )
    }

    // Insert token in storage, SORTED by increasing age ! This allows faster computation of intervals...
    pub fn insert(&mut self, token : TAPNToken) {
        let mut index = self.tokens.len();
        let mut add_to_existing = false;
        for (i, tok) in self.tokens().enumerate() {
            let age = tok.get_age();
            if age > token.age {
                index = i;
                break;
            } else if age == token.age {
                index = i;
                add_to_existing = true;
                break;
            }
        }
        if add_to_existing {
            let existing = TAPNTokenWriter::from(&mut self.tokens[index]);
            *existing.count += token.count;
        } else {
            self.tokens.insert(index, ModelStorage::from(token));
        }
    }

    pub fn remove_set(&mut self, other : &TAPNTokenList) {
        let mut index = 0;
        for to_remove in other.iter() {
            if self.tokens.len() == 0 {
                return;
            }
            while index < self.tokens.len() {
                let token = TAPNTokenWriter::from(&mut self.tokens[index]);
                if token.get_age() == to_remove.age {
                    if *token.count > to_remove.count {
                        *token.count -= to_remove.count;
                        index += 1;
                    } else {
                        self.tokens.remove(index);
                    }
                    break;
                } else {
                    index += 1;
                }
            }
        }
    }

    pub fn get_token(&mut self, index : usize) -> TAPNTokenWriter {
        TAPNTokenWriter::from(&mut self.tokens[index])
    }

    pub fn n_tokens(&self) -> i32 {
        self.tokens.iter().map(|t| *t.ref_tuple().0.ref_int() ).sum()
    }

    pub fn delta(&mut self, dt : ClockValue) {
        for tok in self.tokens() {
            *tok.age += dt.float()
        }
    }

    pub fn get(&self) -> TAPNTokenList {
        self.tokens.iter().map(|t| TAPNToken::from(t.clone()) ).collect()
    }

    pub fn max_age(&self) -> ClockValue {
        if self.tokens.len() == 0 {
            return ClockValue::neg_infinity();
        }
        let last_age = self.tokens.last().unwrap().ref_tuple().1;
        ClockValue::from(*last_age.ref_float())
    }

    pub fn list_len(&self) -> usize {
        self.tokens.len()
    }

}

impl<'a> From<&'a ModelStorage> for TAPNTokenListReader<'a> {
    fn from(value : &'a ModelStorage) -> Self {
        let vec = value.ref_vec();
        TAPNTokenListReader { tokens : vec }
    }
}
impl<'a> From<&'a mut ModelStorage> for TAPNTokenListWriter<'a> {
    fn from(value : &'a mut ModelStorage) -> Self {
        let vec = value.mut_vec();
        TAPNTokenListWriter { tokens : vec }
    }
}

// ----- PlaceList ----- //

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct TAPNPlaceList {
    pub places : Vec<TAPNTokenList>
}
impl TAPNPlaceList {
    pub fn places(n_places : usize) -> TAPNPlaceList {
        TAPNPlaceList {
            places : vec![Vec::new() ; n_places]
        }
    }
    pub fn place_has_token(&self, i_place : usize) -> bool {
        self.places[i_place].len() > 0
    }
}
impl From<ModelStorage> for TAPNPlaceList {
    fn from(value : ModelStorage) -> Self {
        let vec = value.vec();
        let vec = vec.into_iter().map(|x| TAPNTokenList::from(x) ).collect();
        TAPNPlaceList { places : vec }
    }
}
impl From<TAPNPlaceList> for ModelStorage {
    fn from(value : TAPNPlaceList) -> Self {
        let value : Vec<ModelStorage> = value.places.into_iter().map(|t| ModelStorage::from(t) ).collect();
        value.into()
    }
}

pub struct TAPNPlaceListReader<'a> {
    pub places : &'a Vec<ModelStorage>
}
pub struct TAPNPlaceListWriter<'a> {
    pub places : &'a mut Vec<ModelStorage>
}

impl<'a> TAPNPlaceListReader<'a> {

    pub fn n_places(&self) -> usize {
        self.places.len()
    }

    pub fn place(&self, place : usize) -> TAPNTokenListReader {
        TAPNTokenListReader::from(&self.places[place])
    }

}
impl<'a> TAPNPlaceListWriter<'a> {

    pub fn delta(&mut self, dt : ClockValue) {
        for place in self.places.iter_mut() {
            let mut token_list = TAPNPlaceListWriter::from(place);
            token_list.delta(dt)
        }
    }

    pub fn n_places(&self) -> usize {
        self.places.len()
    }

    pub fn place(&mut self, place : usize) -> TAPNTokenListWriter {
        TAPNTokenListWriter::from(&mut self.places[place])
    }

}

impl<'a> From<&'a ModelStorage> for TAPNPlaceListReader<'a> {
    fn from(value : &'a ModelStorage) -> Self {
        let vec = value.ref_vec();
        TAPNPlaceListReader { places : vec }
    }
}
impl<'a> From<&'a mut ModelStorage> for TAPNPlaceListWriter<'a> {
    fn from(value : &'a mut ModelStorage) -> Self {
        let vec = value.mut_vec();
        TAPNPlaceListWriter { places : vec }
    }
}
