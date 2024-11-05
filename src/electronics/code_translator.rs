use crate::models::{model_context::ModelContext, program::Program};

use super::IOContext;

pub trait CodeTranslator {

    fn setup(&mut self, ctx : &ModelContext, io_ctx : &IOContext, hz_rate : f64);

    fn export(&mut self, program : &Program) -> String;

}

pub struct ArduinoExporter {



}

impl CodeTranslator for ArduinoExporter {

    fn setup(&mut self, ctx : &ModelContext, io_ctx : &IOContext, hz_rate : f64) {
        todo!()
    }

    fn export(&mut self, program : &Program) -> String {
        todo!()
    }
    
}