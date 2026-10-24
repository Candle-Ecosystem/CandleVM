use crate::constants::MAX_CAPTURE_COUNT;
use crate::error::{CandleError, CandleResult};
use crate::heap::Object;
use crate::value::Value;
use crate::vm::Vm;

impl Vm {
    pub fn exec_new_closure(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let function_reg = self.fetch_u8()? as usize;
        let capture_count = self.fetch_u8()?;

        if capture_count > MAX_CAPTURE_COUNT {
            return Err(CandleError::InvalidInput(format!(
                "Closure capture count exceeds maximum of {}",
                MAX_CAPTURE_COUNT
            )));
        }

        let mut captures = Vec::with_capacity(capture_count as usize);
        for _ in 0..capture_count {
            let capture_reg = self.fetch_u8()? as usize;
            captures.push(self.get_register(capture_reg));
        }

        let (module_id, function_id) = self.get_register(function_reg).as_function_ref()?;
        let function = self
            .get_module(module_id)
            .ok_or(CandleError::InvalidModuleRef(module_id))?
            .get_function(function_id)
            .ok_or(CandleError::InvalidFunctionRef {
                module_id,
                id: function_id,
            })?;

        if capture_count != function.get_capture_count() {
            return Err(CandleError::InvalidInput(format!(
                "Closure capture count mismatch: function expects {}, bytecode provides {}",
                function.get_capture_count(),
                capture_count
            )));
        }

        let object_id = self
            .get_heap()
            .allocate(Object::Closure(module_id, function_id, captures));
        self.set_register(target_reg, Value::ObjectRef(object_id));
        self.maybe_run_gc();
        Ok(())
    }
}
