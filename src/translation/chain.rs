use crate::{models::{lbl, ModelContext, ModelObject, ModelState}, translation::{Translation, TranslationError, TranslationFactory, TranslationMeta, TranslationResult, TranslationType}};

use TranslationType::*;

pub struct TranslationChain {
    pub translations : Vec<Box<dyn Translation>>
}

impl Translation for TranslationChain {

    fn get_meta(&self) -> TranslationMeta {
        TranslationMeta {
            name : lbl("TranslationChain"),
            description : String::from("Structs used to chain translations into a more complex one."),
            input : match self.translations.first() {
                None => lbl("any"),
                Some(x) => x.get_meta().input
            },
            output : match self.translations.last() {
                None => lbl("any"),
                Some(x) => x.get_meta().output
            },
            translation_type : Unspecified,
        }
    }

    fn translate(&mut self, base : &dyn ModelObject, ctx : &ModelContext, initial_state : &ModelState) -> TranslationResult {
        if self.translations.is_empty() {
            return Err(TranslationError(String::from("Empty translation chain")));
        }
        let mut current_model = base;
        let mut initial_state = initial_state;
        let mut current_ctx = ctx;
        for translation in self.translations.iter_mut() {
            translation.translate(current_model, current_ctx, initial_state)?;
            (current_model, current_ctx, initial_state) = translation.get_translated();
        }
        Ok(())
    }

    fn get_translated(&mut self) -> (&mut dyn ModelObject, &ModelContext, &ModelState) {
        self.translations.last_mut().unwrap().get_translated()
    }

    fn back_translate(&self, state : ModelState) -> Option<ModelState> {
        let mut current_state = state;
        for translation in self.translations.iter().rev() {
            let back = translation.back_translate(current_state);
            match back {
                None => return None,
                Some(s) => current_state = s
            };
        }
        Some(current_state)
    }

    fn forward_translate(&self, state : ModelState) -> Option<ModelState> {
        let mut current_state = state;
        for translation in self.translations.iter() {
            let forward = translation.forward_translate(current_state);
            match forward {
                None => return None,
                Some(s) => current_state = s
            };
        }
        Some(current_state)
    }

}

pub struct TranslationChainFactory {
    pub factories: Vec<Box<dyn TranslationFactory>>
}

impl TranslationFactory for TranslationChainFactory {

    fn get_meta(&self) -> TranslationMeta {
        TranslationMeta {
            name : lbl("TranslationChain"),
            description : String::from("Structs used to chain translations into a more complex one."),
            input : match self.factories.first() {
                None => lbl("any"),
                Some(x) => x.get_meta().input
            },
            output : match self.factories.last() {
                None => lbl("any"),
                Some(x) => x.get_meta().output
            },
            translation_type : Unspecified,
        }
    }
    
    fn make_instance(&self) -> Box<dyn Translation> {
        Box::new(TranslationChain {
            translations : self.factories.iter().map(|t| t.make_instance()).collect()
        })
    
    }
    
}
