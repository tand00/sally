use super::Model;

pub struct Translator<T : Model, U : Model> {
    pub base_model : T,
    pub translated_model : Option<U>,
}

impl<T : Model, U : Model> Translator<T, U> {
    
    pub fn new(base : T) -> Self {
        Translator {
            base_model : base,
            translated_model : None
        }
    }

}