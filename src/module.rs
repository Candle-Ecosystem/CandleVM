use crate::function::Function;
use std::collections::HashMap;

pub struct Module {
    name: String,
    functions: HashMap<u16, Function>,
}

impl Module {
    pub fn new(name: String, functions: HashMap<u16, Function>) -> Self {
        Self {
            name,
            functions,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_functions(&self) -> &HashMap<u16, Function> {
        &self.functions
    }

    pub fn get_function(&self, id: u16) -> Option<&Function> {
        for (function_id, function) in &self.functions {
            if *function_id == id {
                return Some(function);
            }
        }
        None
    }

    pub fn get_function_id(&self, name: String) -> Option<u16> {
        for (function_id, function) in &self.functions {
            if *function.get_name() == name {
                return Some(*function_id);
            }
        }
        None
    }
}
