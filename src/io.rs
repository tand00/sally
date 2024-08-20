use crate::models::{Label, ModelObject};

pub mod pnml;
pub mod tapn;
pub mod sly;

pub struct ModelLoaderMeta {
    pub name : Label,
    pub description : String,
    pub ext : Label,
    pub output : Label
}

pub trait ModelLoader {

    fn get_meta(&self) -> ModelLoaderMeta;

    fn load(&self, path : String) -> Box<dyn ModelObject>;

}