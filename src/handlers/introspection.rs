use crate::error::CandleResult;
use crate::value::Value;
use crate::vm::Vm;

impl Vm {
    pub fn exec_type(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let reg = self.fetch_u8()? as usize;
        let val = self.get_register(reg);
        self.set_register(target_reg, Value::Byte(val.value_type() as u8));
        Ok(())
    }

    pub fn exec_object_type(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;
        let ref_val = self.get_register(ref_reg).as_object_ref()?;

        let object_type = {
            let object = self.get_heap().get(ref_val)?;
            object.object_type() as u8
        };

        self.set_register(target_reg, Value::Byte(object_type));
        Ok(())
    }
}
