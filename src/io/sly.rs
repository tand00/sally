use std::collections::HashMap;

use serde_json::{Map, Value};

use crate::models::{lbl, markov::markov_chain::MarkovChain, ModelProject, petri::{PetriNet, PetriStructure}, tapn::{TAPNStructure, TAPN}, Label, Model, ModelObject};

use super::{deserialize_structure, serialize_structure, ModelIOError, ModelLoader, ModelLoaderMeta, ModelLoadingResult, ModelWriter, ModelWriterMeta, ModelWritingResult};

pub struct SLYLoader;
pub struct SLYWriter;

const MODEL_TYPE_KEY : &str = "model-type";
const MODEL_KEY : &str = "model";
const INITIAL_STATE_KEY : &str = "initial-state";
const QUERIES_KEY : &str = "queries";

impl SLYLoader {

    pub fn load_model(model_type : Label, serialized : Value) -> Result<Box<dyn ModelObject>, ModelIOError> {
        if model_type == PetriNet::get_meta().name {
            deserialize_structure::<PetriStructure, PetriNet>(serialized)
        } else if model_type == TAPN::get_meta().name {
            deserialize_structure::<TAPNStructure, TAPN>(serialized)
        } else if model_type == MarkovChain::get_meta().name {
            let chain : MarkovChain = serde_json::from_value(serialized)?;
            Ok(Box::new(chain))
        } else {
            Err(ModelIOError)
        }
    }
}

impl SLYWriter {

    pub fn write_model(model : &dyn ModelObject) -> Result<Value, ModelIOError> {
        let model_type = model.get_model_meta().name;
        if model_type == PetriNet::get_meta().name {
            serialize_structure::<PetriNet, PetriStructure>(&*model)
        } else if model_type == TAPN::get_meta().name {
            serialize_structure::<TAPN, TAPNStructure>(&*model)
        } else if model_type == MarkovChain::get_meta().name {
            let Some(chain) = model.as_any().downcast_ref::<MarkovChain>() else {
                return Err(ModelIOError);
            };
            Ok(serde_json::to_value(chain.clone())?)
        } else {
            Err(ModelIOError)
        }
    }

}

impl ModelLoader for SLYLoader {

    fn get_meta(&self) -> ModelLoaderMeta {
        ModelLoaderMeta { 
            name: lbl("SLYLoader"),
            description: "Generic loader to load SLY files, that embed metada to identify the model type".to_owned(), 
            ext: lbl("sly"), 
            output: lbl("any")
        }
    }

    fn load(&self, content : String) -> ModelLoadingResult {
        let parsed = serde_json::from_str::<Value>(&content)?;

        let Value::Object(mut map) = parsed else {
            return Err(ModelIOError)
        };
        let Value::String(model_type) = map[MODEL_TYPE_KEY].clone() else {
            return Err(ModelIOError)
        };
        
        let model_type = Label::from(model_type);
        let Some(serialized) = map.remove(MODEL_KEY) else {
            return Err(ModelIOError)
        };

        let initial = map.remove(INITIAL_STATE_KEY);
        let initial = if initial.is_some() {
            serde_json::from_value(initial.unwrap())?
        } else {
            HashMap::new()
        };

        let queries = map.remove(QUERIES_KEY);
        let queries = if queries.is_some() {
            serde_json::from_value(queries.unwrap())?
        } else {
            Vec::new()
        };

        let model = SLYLoader::load_model(model_type, serialized)?;

        Ok(ModelProject::new(
            model, queries, initial
        ))
    }

}

impl ModelWriter for SLYWriter {

    fn get_meta(&self) -> ModelWriterMeta {
        ModelWriterMeta {
            name: lbl("SLYWriter"),
            description: "Generic writer to write SLY files, that embed metada to identify the model type".to_owned(),
            ext: lbl("sly"),
            input: lbl("any"),
        }
    }

    fn write(&self, project : &ModelProject) -> ModelWritingResult {
        let model_type = Value::String(project.model.get_model_meta().name.to_string());
        let value = Self::write_model(&*project.model)?;
        let mut map = Map::new();
        map.insert(MODEL_TYPE_KEY.to_owned(), model_type);
        map.insert(MODEL_KEY.to_owned(), value);
        if !project.initial_marking.is_empty() {
            let initial = serde_json::to_value(project.initial_marking.clone())?;
            map.insert(INITIAL_STATE_KEY.to_owned(), initial);
        }
        if !project.queries.is_empty() {
            let queries = serde_json::to_value(project.queries.clone())?;
            map.insert(QUERIES_KEY.to_owned(), queries);
        }
        Ok(serde_json::to_string(&map)?)
    }

}