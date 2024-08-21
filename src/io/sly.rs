use serde_json::{Map, Value};

use crate::models::{lbl, markov::markov_chain::MarkovChain, petri::{PetriNet, PetriStructure}, tapn::{TAPNStructure, TAPN}, Label, Model, ModelObject};

use super::{deserialize_structure, serialize_structure, ModelIOError, ModelLoader, ModelLoaderMeta, ModelLoadingResult, ModelWriter, ModelWriterMeta, ModelWritingResult};

pub struct SLYLoader;
pub struct SLYWriter;

impl SLYLoader {

    pub fn load_model(model_type : Label, serialized : Value) -> ModelLoadingResult {
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
        let Value::String(model_type) = map["model-type"].clone() else {
            return Err(ModelIOError)
        };
        
        let model_type = Label::from(model_type);
        let Some(serialized) = map.remove("model") else {
            return Err(ModelIOError)
        };

        SLYLoader::load_model(model_type, serialized)
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

    fn write(&self, model : &dyn ModelObject) -> ModelWritingResult {
        let model_tye = Value::String(model.get_model_meta().name.to_string());
        let value = Self::write_model(model)?;
        let mut map = Map::new();
        map.insert("model-type".to_owned(), model_tye);
        map.insert("model".to_owned(), value);
        Ok(serde_json::to_string(&map)?)
    }

}