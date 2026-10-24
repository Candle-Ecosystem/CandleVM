use crate::callstack::CallFrame;
use crate::constants::{MAX_CALL_STACK, MAX_FUNCTION_PARAMETERS};
use crate::error::{CandleError, CandleResult};
use crate::heap::{Object, ObjectType};
use crate::value::Value;
use crate::vm::Vm;
use std::borrow::Cow;

impl Vm {
    pub fn exec_nop(&self) -> CandleResult<()> {
        Ok(())
    }

    pub fn exec_exit(&mut self) -> CandleResult<()> {
        let code_reg = self.fetch_u8()? as usize;
        let code_val = self.get_register(code_reg);
        self.stop(code_val.as_u8()?);
        Ok(())
    }

    pub fn exec_call(&mut self) -> CandleResult<()> {
        let return_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;
        self.exec_call_internal(return_reg, None, ref_reg)
    }

    pub fn exec_pcall(&mut self) -> CandleResult<()> {
        let return_reg = self.fetch_u8()? as usize;
        let error_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;
        self.exec_call_internal(return_reg, Some(error_reg), ref_reg)
    }

    fn exec_call_internal(
        &mut self,
        return_reg: usize,
        error_reg: Option<usize>,
        ref_reg: usize,
    ) -> CandleResult<()> {
        if self.get_call_stack().len() >= MAX_CALL_STACK {
            return Err(CandleError::CallstackOverflow);
        }

        let (function_module, function_id, captures) = match self.get_register(ref_reg) {
            Value::ObjectRef(object_id) => {
                let object = self.get_heap().get(object_id)?;
                match object {
                    Object::Closure(module_id, function_id, captures) => {
                        (*module_id, *function_id, Some(captures.clone()))
                    }
                    _ => {
                        return Err(CandleError::ObjectTypeMismatch {
                            expected: Cow::Borrowed(&[ObjectType::Closure]),
                            found: object.object_type(),
                        });
                    }
                }
            }
            value => {
                let (module_id, function_id) = value.as_function_ref()?;
                (module_id, function_id, None)
            }
        };
        let (function_parameter_count, handler, native) = {
            let module = self
                .get_module(function_module)
                .ok_or(CandleError::InvalidModuleRef(function_module))?;

            let function =
                module
                    .get_function(function_id)
                    .ok_or(CandleError::InvalidFunctionRef {
                        module_id: function_module,
                        id: function_id,
                    })?;

            (
                function.get_parameter_count(),
                function.get_handler(),
                function.is_native(),
            )
        };

        let captures = captures.unwrap_or_else(|| Vec::new());

        let supplied_parameter_count = self.fetch_u8()?;
        if supplied_parameter_count > MAX_FUNCTION_PARAMETERS {
            return Err(CandleError::InvalidInput(format!(
                "Call parameter count exceeds maximum of {}",
                MAX_FUNCTION_PARAMETERS
            )));
        }

        let mut parameters = Vec::with_capacity(function_parameter_count as usize);
        for index in 0..supplied_parameter_count {
            let parameter_reg = self.fetch_u8()? as usize;
            let value = self.get_register(parameter_reg);
            if index < function_parameter_count {
                parameters.push(value);
            }
        }
        parameters.resize(function_parameter_count as usize, Value::Null);
        parameters.extend(captures);

        let parameters = parameters.as_slice();

        let base = self.get_registers().len();
        let pc = self.get_pc();

        self.get_call_stack().push(CallFrame::new(
            function_module,
            function_id,
            pc,
            return_reg,
            error_reg,
            base,
            native,
        ));

        match handler(self, parameters) {
            Ok(value_opt) => {
                if let Some(value) = value_opt {
                    match self.get_call_stack().pop() {
                        Some(frame) => {
                            self.set_register(frame.get_return_register(), value);
                            Ok(())
                        }
                        None => Err(CandleError::CallstackUnderflow),
                    }?;
                    Ok(())
                } else {
                    Ok(())
                }
            }
            Err(err) => Err(err),
        }
    }

    pub fn exec_ret(&mut self) -> CandleResult<()> {
        let return_reg = self.fetch_u8()? as usize;
        let return_val = self.get_register(return_reg);

        match self.get_call_stack().pop() {
            Some(frame) => {
                self.get_registers().truncate(frame.get_register_base());
                self.set_register(frame.get_return_register(), return_val);
                self.set_pc(frame.get_return_pc());
                Ok(())
            }
            None => Err(CandleError::CallstackUnderflow),
        }?;

        Ok(())
    }

    pub fn exec_throw(&mut self) -> CandleResult<()> {
        let ref_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;
        let error = {
            let object = self.get_heap().get(ref_val)?;
            match object {
                Object::Error(err) => err.clone(),
                _ => {
                    return Err(CandleError::ObjectTypeMismatch {
                        expected: Cow::Borrowed(&[ObjectType::Error]),
                        found: object.object_type(),
                    });
                }
            }
        };

        self.handle_error(error);
        Ok(())
    }

    pub fn exec_jump(&mut self) -> CandleResult<()> {
        let offset = self.fetch_i32()?;

        let new_pc = (self.get_pc() as i64) + offset as i64;
        if new_pc < 0 || new_pc >= self.get_bytecode().len() as i64 {
            Err(CandleError::InvalidJump(new_pc as usize))
        } else {
            self.set_pc(new_pc as usize);
            Ok(())
        }
    }

    pub fn exec_jump_if(&mut self) -> CandleResult<()> {
        let offset = self.fetch_i32()?;
        let condition_reg = self.fetch_u8()? as usize;

        let condition_val = self.get_register(condition_reg);
        if condition_val.as_bool()? {
            let new_pc = (self.get_pc() as i64) + offset as i64;
            return if new_pc < 0 || new_pc >= self.get_bytecode().len() as i64 {
                Err(CandleError::InvalidJump(new_pc as usize))
            } else {
                self.set_pc(new_pc as usize);
                Ok(())
            };
        }
        Ok(())
    }

    pub fn exec_jump_if_not(&mut self) -> CandleResult<()> {
        let offset = self.fetch_i32()?;
        let condition_reg = self.fetch_u8()? as usize;

        let condition_val = self.get_register(condition_reg);
        if !condition_val.as_bool()? {
            let new_pc = (self.get_pc() as i64) + offset as i64;
            return if new_pc < 0 || new_pc >= self.get_bytecode().len() as i64 {
                Err(CandleError::InvalidJump(new_pc as usize))
            } else {
                self.set_pc(new_pc as usize);
                Ok(())
            };
        }
        Ok(())
    }
}
