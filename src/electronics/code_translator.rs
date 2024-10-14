use crate::models::program;

pub trait CodeTranslator {

    fn setup(&mut self, ctx : &ModelContext, io_ctx : &IOContext);

    fn export(&mut self, program : &Program) -> String;

}