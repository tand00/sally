use crate::models::{model_storage::ModelStorage, time::ClockValue};

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub struct TAPNToken {
    pub count : i32,
    pub age : ClockValue
}

pub type TAPNTokenSet = Vec<(usize, TAPNToken)>;

pub struct TAPNTokenAccessor<'a> {
    pub count : &'a mut i32,
    pub age : &'a mut f64
}

impl<'a> TAPNTokenAccessor<'a> {
    
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
impl<'a> From<&'a mut ModelStorage> for TAPNTokenAccessor<'a> {
    fn from(value: &'a mut ModelStorage) -> Self {
        let (count, age) = value.mut_tuple();
        let count = count.mut_int();
        let age = age.mut_float();
        TAPNTokenAccessor { count, age }
    }
}
impl From<TAPNToken> for ModelStorage {
    fn from(value: TAPNToken) -> Self {
        let count = ModelStorage::from(value.count);
        let age = ModelStorage::from(value.age.float());
        (count, age).into()
    }
}

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

pub struct TAPNTokenListAccessor<'a> {
    pub tokens: &'a mut Vec<ModelStorage> //Vec<TAPNTokenAccessor<'a>>
}

impl<'a> TAPNTokenListAccessor<'a> {

    pub fn tokens(&mut self) -> Vec<TAPNTokenAccessor> {
        self.tokens.iter_mut().map(|x| TAPNTokenAccessor::from(x) ).collect()
    }

    // Insert token in storage, SORTED by increasing age ! This allows faster computation of intervals...
    pub fn insert(&mut self, token : TAPNToken) {
        let mut index = self.tokens.len();
        let mut add_to_existing = false;
        for (i, tok) in self.tokens().iter().enumerate() {
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
            *self.tokens()[index].count += token.count;
        } else {
            self.tokens.insert(index, ModelStorage::from(token));
        }
    }

    pub fn n_tokens(&self) -> i32 {
        self.tokens.iter().map(|t| *t.ref_tuple().0.ref_int() ).sum()
    }

    pub fn delta(&mut self, dt : ClockValue) {
        for tok in self.tokens().iter_mut() {
            *tok.age += dt.float()
        }
    }

    pub fn get(&self) -> TAPNTokenList {
        self.tokens.iter().map(|t| TAPNToken::from(t.clone()) ).collect()
    }

}

impl<'a> From<&'a mut ModelStorage> for TAPNTokenListAccessor<'a> {
    fn from(value : &'a mut ModelStorage) -> Self {
        let vec = value.mut_vec();
        TAPNTokenListAccessor { tokens : vec }
    }
}

#[derive(Debug, Clone, Hash, PartialEq)]
pub struct TAPNPlaceList {
    pub places : Vec<TAPNTokenList>
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

pub struct TAPNPlaceListAccessor<'a> {
    pub places : Vec<TAPNTokenListAccessor<'a>>
}

impl<'a> TAPNPlaceListAccessor<'a> {

    pub fn delta(&mut self, dt : ClockValue) {
        for place in self.places.iter_mut() {
            place.delta(dt)
        }
    }

}

impl<'a> From<&'a mut ModelStorage> for TAPNPlaceListAccessor<'a> {
    fn from(value : &'a mut ModelStorage) -> Self {
        let vec = value.mut_vec();
        let vec = vec.iter_mut().map(|x| TAPNTokenListAccessor::from(x) ).collect();
        TAPNPlaceListAccessor { places : vec }
    }
}