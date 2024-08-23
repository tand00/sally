use std::{fs, io};

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::models::{InitialMarking, Label, ModelObject};

pub mod pnml;
pub mod tapn;
pub mod sly;

pub struct ModelIOError;
trait ModelIOErrorVariant {}
impl ModelIOErrorVariant for serde_json::Error {}
impl ModelIOErrorVariant for io::Error {}
impl<T : ModelIOErrorVariant> From<T> for ModelIOError {
    fn from(_ : T) -> Self { Self }
}

pub type LoadedModel = (Box<dyn ModelObject>, Option<InitialMarking>);

pub type ModelLoadingResult = Result<LoadedModel, ModelIOError>;
pub type ModelWritingResult = Result<String, ModelIOError>;

pub struct ModelLoaderMeta {
    pub name : Label,
    pub description : String,
    pub ext : Label,
    pub output : Label
}

pub struct ModelWriterMeta {
    pub name : Label,
    pub description : String,
    pub ext : Label,
    pub input : Label
}

pub trait ModelLoader {

    fn get_meta(&self) -> ModelLoaderMeta;

    fn load(&self, content : String) -> ModelLoadingResult;

    fn load_file(&self, path : String) -> ModelLoadingResult {
        let content = fs::read_to_string(path)?;
        self.load(content)
    }

}

pub trait ModelWriter {

    fn get_meta(&self) -> ModelWriterMeta;

    fn write(&self, model : &dyn ModelObject, initial : Option<InitialMarking>) -> ModelWritingResult;

    fn write_file(&self, path : String, model : &dyn ModelObject, initial : Option<InitialMarking>) -> ModelWritingResult {
        let content = self.write(model, initial)?;
        fs::write(path, content.clone())?;
        Ok(content)
    }

}

pub fn deserialize_structure<T, U>(serialized : Value) -> Result<Box<dyn ModelObject>, ModelIOError> 
    where 
        T : DeserializeOwned,
        U : ModelObject + From<T>
{
    let structure = serde_json::from_value::<T>(serialized)?;
    Ok(Box::new(U::from(structure)))
}

pub fn serialize_structure<'a, T, U>(model : &'a dyn ModelObject) -> Result<Value, ModelIOError> 
    where 
        T : ModelObject,
        U : Serialize + From<&'a T>
{
    let Some(downcasted) = model.as_any().downcast_ref::<T>() else {
        return Err(ModelIOError);
    };
    let structure = U::from(downcasted);
    let value = serde_json::to_value(structure)?;
    Ok(value)
}