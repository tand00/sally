use serde::{Deserialize, Serialize};

use crate::verification::Verifiable;

use super::{model_context::ModelContext, Label, ModelState};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct MappingError(pub Label);
impl Display for MappingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mapping error : label {} not found in context", self.0)
    }
}
pub type MappingResult<T> = Result<T, MappingError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum VarType {
    UnknownType,
    VarU8, VarI8,
    VarU16, VarI16,
    VarU32, VarI32
}

impl VarType {
    pub fn size(&self) -> usize {
        match self {
            Self::UnknownType => 0,
            Self::VarU8 | Self::VarI8 => 1,
            Self::VarU16 | Self::VarI16 => 2,
            Self::VarU32 | Self::VarI32 => 4
        }
    }
    pub fn is_unknown(&self) -> bool {
        return *self == Self::UnknownType
    }
}

impl Default for VarType {
    fn default() -> Self {
        Self::UnknownType
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct ModelVar {
    pub name : Label,
    #[serde(skip)]
    var_type : VarType,
    #[serde(skip)]
    address : Option<usize>,
}

impl ModelVar {

    pub fn new() -> ModelVar {
        ModelVar { 
            name: Label::new(), 
            var_type: VarType::UnknownType, 
            address: None 
        }
    }

    pub fn name(name : Label) -> ModelVar {
        ModelVar { name, address : None, var_type : VarType::UnknownType }
    }

    pub fn address(index : usize, var_type : VarType) -> ModelVar {
        if var_type.is_unknown() {
            panic!("Impossible to define a variable address before setting its type !")
        }
        ModelVar { name : Label::new(), address : Some(index), var_type }
    }

    pub fn make_defined(name : Label, address : usize, var_type : VarType) -> ModelVar {
        if var_type.is_unknown() {
            panic!("Impossible to define a variable address before setting its type !")
        }
        ModelVar { name, address : Some(address), var_type }
    }

    pub fn get_name(&self) -> Label {
        self.name.clone()
    }

    pub fn size(&self) -> usize {
        self.var_type.size()
    }

    pub fn is_mapped(&self) -> bool {
        self.address.is_some()
    }

    pub fn get_address(&self) -> usize {
        self.address.unwrap()
    }

    pub fn set_address(&mut self, address : usize) {
        if self.var_type.is_unknown() {
            panic!("Impossible to define a variable address before setting its type !")
        }
        self.address = Some(address)
    }

    pub fn get_type(&self) -> VarType {
        self.var_type
    }

    pub fn set_type(&mut self, var_type : VarType) {
        self.var_type = var_type
    }

    pub fn apply_to(self, ctx : &ModelContext) -> MappingResult<ModelVar> {
        let res = ctx.get_var(&self.name);
        match res {
            None => Err(MappingError(Label::from("Unable to map var to index !"))),
            Some(v) => Ok(v)
        }
    }

    pub fn evaluate(&self, state : &impl Verifiable) -> i32 {
        if self.address.is_none() {
            panic!("Can't evaluate unmapped var !");
        }
        state.evaluate_var(&self)
    }

    pub fn set(&self, state : &mut ModelState, value : i32) {
        if self.address.is_none() {
            panic!("Can't set unmapped var !");
        }
        state.set_marking(&self, value);
    }
    
    pub fn unbind(&mut self) {
        self.address = None;
        self.var_type = VarType::UnknownType;
    }

}

impl<T : Into<String>> From<T> for ModelVar {
    fn from(value: T) -> Self {
        ModelVar::name(Label::from(value))
    }
}

pub fn var(name : &str) -> ModelVar {
    ModelVar::name(Label::from(name))
}

impl Default for ModelVar {

    fn default() -> Self {
        ModelVar::new()
    }

}