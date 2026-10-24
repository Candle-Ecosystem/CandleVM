use crate::error::{CandleError, CandleResult};
use crate::heap::Object;
use crate::value::Value;
use crate::vm::Vm;

impl Vm {
    pub fn exec_load_const(&mut self) -> CandleResult<()> {
        let reg = self.fetch_u8()? as usize;
        let val = self.fetch_value()?;
        self.set_register(reg, val);
        Ok(())
    }

    pub fn exec_mov(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let source_reg = self.fetch_u8()? as usize;
        let val = self.get_register(source_reg);
        self.set_register(target_reg, val);
        Ok(())
    }

    pub fn exec_load_module(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let name = self.fetch_string()?;

        if let Some(id) = self.get_module_id(name.clone()) {
            self.set_register(target_reg, Value::ModuleRef(id));
            Ok(())
        } else {
            Err(CandleError::ModuleName(name))
        }
    }

    pub fn exec_load_function(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let module_ref = self.fetch_u8()? as usize;
        let name = self.fetch_string()?;

        let module_ref = self.get_register(module_ref).as_module_ref()?;
        let module_opt = self.get_module(module_ref);
        if let Some(module) = module_opt {
            match module.get_function_id(name.clone()) {
                Some(function) => {
                    self.set_register(target_reg, Value::FunctionRef(module_ref, function));
                    Ok(())
                }
                None => Err(CandleError::FunctionName {
                    module: module.get_name().clone(),
                    name,
                }),
            }
        } else {
            Err(CandleError::InvalidModuleRef(module_ref))
        }
    }

    pub fn exec_load_global(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let global = self.fetch_string()?;
        let val = self.get_global(global);
        self.set_register(target_reg, val);
        Ok(())
    }

    pub fn exec_store_global(&mut self) -> CandleResult<()> {
        let target_global = self.fetch_string()?;
        let source_reg = self.fetch_u8()? as usize;
        let val = self.get_register(source_reg);
        self.set_global(target_global, val, false)?;
        Ok(())
    }

    pub fn exec_load_resource(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let id = self.fetch_string()?;

        let resource = self
            .get_resources()
            .get(&id)
            .cloned()
            .ok_or(CandleError::ResourceNotFound(id))?;

        let object_id = self.get_heap().allocate(Object::Buffer(resource));

        self.set_register(target_reg, Value::ObjectRef(object_id));
        Ok(())
    }
}
