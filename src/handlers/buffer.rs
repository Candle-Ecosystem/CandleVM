use crate::error::{CandleError, CandleResult};
use crate::heap::{Object, ObjectType};
use crate::value::Value;
use crate::vm::Vm;
use std::borrow::Cow;

impl Vm {
    pub fn exec_new_buffer(&mut self) -> CandleResult<()> {
        let reg = self.fetch_u8()? as usize;
        let size_reg = self.fetch_u8()? as usize;

        let size_val = self.get_register(size_reg);
        let size = size_val.as_usize()?;

        let id = self.get_heap().allocate(Object::Buffer(vec![0u8; size]));
        self.set_register(reg, Value::ObjectRef(id));
        self.maybe_run_gc();
        Ok(())
    }

    pub fn exec_buffer_len(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;
        let len = {
            let object = self.get_heap().get(ref_val)?;
            match object {
                Object::Buffer(vec) => vec.len(),
                _ => {
                    return Err(CandleError::ObjectTypeMismatch {
                        expected: Cow::Borrowed(&[ObjectType::Buffer]),
                        found: object.object_type(),
                    });
                }
            }
        };

        self.set_register(target_reg, Value::from_f64(len as f64)?);
        Ok(())
    }

    pub fn exec_buffer_get(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;
        let index_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;
        let index_val = self.get_register(index_reg);

        let (val, len) = {
            let object = self.get_heap().get(ref_val)?;
            match object {
                Object::Buffer(vec) => (vec.get(index_val.as_usize()?).cloned(), vec.len()),
                _ => {
                    return Err(CandleError::ObjectTypeMismatch {
                        expected: Cow::Borrowed(&[ObjectType::Buffer]),
                        found: object.object_type(),
                    });
                }
            }
        };

        if let Some(val) = val {
            self.set_register(target_reg, Value::Byte(val));
            Ok(())
        } else {
            Err(CandleError::BufferIndexOutOfBounds {
                index: index_val.as_usize()?,
                len,
            })
        }
    }

    pub fn exec_buffer_set(&mut self) -> CandleResult<()> {
        let ref_reg = self.fetch_u8()? as usize;
        let index_reg = self.fetch_u8()? as usize;
        let value_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;
        let index_val = self.get_register(index_reg);
        let value = self.get_register(value_reg);

        let object = self.get_heap().get_mut(ref_val)?;

        match object {
            Object::Buffer(vec) => {
                let index = index_val.as_usize()?;
                if index >= vec.len() {
                    return Err(CandleError::BufferIndexOutOfBounds {
                        index,
                        len: vec.len(),
                    });
                }

                vec[index] = value.as_u8()?;
                Ok(())
            }
            _ => Err(CandleError::ObjectTypeMismatch {
                expected: Cow::Borrowed(&[ObjectType::Buffer]),
                found: object.object_type(),
            }),
        }
    }
}
