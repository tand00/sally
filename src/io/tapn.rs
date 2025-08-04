use crate::models::{lbl, ModelProject, tapn::TAPN, Model};

use super::{ModelLoader, ModelLoaderMeta, ModelLoadingResult, ModelWriter, ModelWriterMeta, ModelWritingResult};

pub struct TAPNLoader;

impl ModelLoader for TAPNLoader {
    fn get_meta(&self) -> ModelLoaderMeta {
        ModelLoaderMeta {
            name : lbl("TAPNLoader"),
            description : "Timed-Arcs Petri nets loader from .tapn files".to_owned(),
            ext : lbl("tapn"),
            output : TAPN::get_meta().name
        }
    }

    fn load(&self, content : String) -> ModelLoadingResult {
        todo!()
    }
}

pub struct TAPNWriter;

impl ModelWriter for TAPNWriter {

    fn get_meta(&self) -> ModelWriterMeta {
        ModelWriterMeta { 
            name: lbl("TAPNWriter"), 
            description: "Timed-Arcs Petri nets writer to .tapn files".to_owned(), 
            ext: lbl("tapn"), 
            input: TAPN::get_meta().name
        }
    }

    fn write(&self, project : &ModelProject) -> ModelWritingResult {
        todo!()
    }

}