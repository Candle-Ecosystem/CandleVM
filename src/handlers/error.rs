use crate::error::{CandleError, CandleResult};
use crate::heap::{Object, ObjectType};
use crate::value::Value;
use crate::vm::Vm;
use std::borrow::Cow;

impl Vm {
    pub fn exec_new_error(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;

        let message = {
            let object = self.get_heap().get(ref_val)?;
            match object {
                Object::String(string) => string,
                _ => {
                    return Err(CandleError::ObjectTypeMismatch {
                        expected: Cow::Borrowed(&[ObjectType::String]),
                        found: object.object_type(),
                    });
                }
            }
        };

        let error = CandleError::Custom(message.into());
        let id = self.get_heap().allocate(Object::Error(error));

        self.set_register(target_reg, Value::ObjectRef(id));
        self.maybe_run_gc();
        Ok(())
    }

    pub fn exec_error_type(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;

        let error_type = {
            let object = self.get_heap().get(ref_val)?;

            match object {
                Object::Error(err) => err.get_type().into(),
                _ => {
                    return Err(CandleError::ObjectTypeMismatch {
                        expected: Cow::Borrowed(&[ObjectType::Error]),
                        found: object.object_type(),
                    });
                }
            }
        };

        let id = self.get_heap().allocate(Object::String(error_type));

        self.set_register(target_reg, Value::ObjectRef(id));
        Ok(())
    }

    pub fn exec_error_message(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;

        let error_message = {
            let object = self.get_heap().get(ref_val)?;
            match object {
                Object::Error(err) => err.get_message(),
                _ => {
                    return Err(CandleError::ObjectTypeMismatch {
                        expected: Cow::Borrowed(&[ObjectType::Error]),
                        found: object.object_type(),
                    });
                }
            }
        };

        let id = self.get_heap().allocate(Object::String(error_message));

        self.set_register(target_reg, Value::ObjectRef(id));
        Ok(())
    }
}
