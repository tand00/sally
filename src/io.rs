use crate::models::{Label, Model};

pub mod pnml;

pub struct ModelLoaderMeta {
    pub name : Label,
    pub description : String,
    pub ext : Label,
    pub output : Label
}

pub trait ModelLoader {

    fn get_meta(&self) -> ModelLoaderMeta;

    fn load(&self, path : String) -> Box<dyn Model>;

}