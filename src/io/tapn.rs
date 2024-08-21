use crate::models::{lbl, tapn::TAPN, Model};

use super::{ModelLoader, ModelLoaderMeta, ModelLoadingResult};

pub struct TAPNLoader {

}

impl ModelLoader for TAPNLoader {
    fn get_meta(&self) -> super::ModelLoaderMeta {
        ModelLoaderMeta {
            name : lbl("TAPNLoader"),
            description : "Timed-Arcs Petri nets loader from .tapn files".to_owned(),
            ext : lbl("tapn"),
            output : TAPN::get_meta().name
        }
    }

    fn load(&self, path : String) -> ModelLoadingResult {
        todo!()
    }
}