use crate::error::{CandleError, CandleResult};
use crate::value::{Value, ValueType};
use crate::vm::Vm;
use std::borrow::Cow;

impl Vm {
    pub fn exec_eq(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, Value::Bool(a_val == b_val)))
    }

    pub fn exec_ne(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, Value::Bool(a_val != b_val)))
    }

    pub fn exec_object_eq(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_value = self.get_register(a_reg);
        let b_value = self.get_register(b_reg);

        let a_id = match a_value {
            Value::ObjectRef(id) => id,
            _ => {
                return Err(CandleError::ValueTypeMismatch {
                    expected: Cow::Borrowed(&[ValueType::ObjectRef]),
                    found: a_value.value_type(),
                });
            }
        };

        let b_id = match b_value {
            Value::ObjectRef(id) => id,
            _ => {
                return Err(CandleError::ValueTypeMismatch {
                    expected: Cow::Borrowed(&[ValueType::ObjectRef]),
                    found: b_value.value_type(),
                });
            }
        };

        let equals = {
            let heap = self.get_heap();
            heap.get(a_id)? == heap.get(b_id)?
        };

        self.set_register(target_reg, Value::Bool(equals));
        Ok(())
    }

    pub fn exec_object_ne(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_value = self.get_register(a_reg);
        let b_value = self.get_register(b_reg);

        let a_id = match a_value {
            Value::ObjectRef(id) => id,
            _ => {
                return Err(CandleError::ValueTypeMismatch {
                    expected: Cow::Borrowed(&[ValueType::ObjectRef]),
                    found: a_value.value_type(),
                });
            }
        };

        let b_id = match b_value {
            Value::ObjectRef(id) => id,
            _ => {
                return Err(CandleError::ValueTypeMismatch {
                    expected: Cow::Borrowed(&[ValueType::ObjectRef]),
                    found: b_value.value_type(),
                });
            }
        };

        let are_different = {
            let heap = self.get_heap();
            heap.get(a_id)? != heap.get(b_id)?
        };

        self.set_register(target_reg, Value::Bool(are_different));
        Ok(())
    }

    pub fn exec_lt(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, Value::Bool(a_val < b_val)))
    }

    pub fn exec_lte(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, Value::Bool(a_val <= b_val)))
    }

    pub fn exec_gt(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, Value::Bool(a_val > b_val)))
    }

    pub fn exec_gte(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;

        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_val = self.get_register(a_reg);
        let b_val = self.get_register(b_reg);

        Ok(self.set_register(target_reg, Value::Bool(a_val >= b_val)))
    }
}
