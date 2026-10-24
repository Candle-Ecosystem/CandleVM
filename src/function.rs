use crate::error::CandleResult;
use crate::value::Value;
use crate::vm::Vm;
use std::sync::Arc;

#[derive(Clone)]
pub struct Function {
    name: String,
    parameter_count: u8,
    capture_count: u8,
    handler: Arc<dyn Fn(&mut Vm, &[Value]) -> CandleResult<Option<Value>>>,
    native: bool,
}

impl Function {
    pub fn new(
        name: String,
        parameter_count: u8,
        handler: Arc<dyn Fn(&mut Vm, &[Value]) -> CandleResult<Option<Value>>>,
        native: bool,
    ) -> Self {
        Self::new_with_captures(name, parameter_count, 0, handler, native)
    }

    pub fn new_with_captures(
        name: String,
        parameter_count: u8,
        capture_count: u8,
        handler: Arc<dyn Fn(&mut Vm, &[Value]) -> CandleResult<Option<Value>>>,
        native: bool,
    ) -> Self {
        Self {
            name,
            parameter_count,
            capture_count,
            handler,
            native,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_parameter_count(&self) -> u8 {
        self.parameter_count
    }

    pub fn get_capture_count(&self) -> u8 {
        self.capture_count
    }

    pub fn get_handler(&self) -> Arc<dyn Fn(&mut Vm, &[Value]) -> CandleResult<Option<Value>>> {
        Arc::clone(&self.handler)
    }

    pub fn is_native(&self) -> bool {
        self.native
    }
}
