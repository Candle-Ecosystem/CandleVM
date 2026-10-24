use crate::error::{CandleError, CandleResult};
use crate::heap::{Object, ObjectType};
use crate::value::Value;
use crate::vm::Vm;
use std::borrow::Cow;
use std::collections::HashMap;

impl Vm {
    pub fn exec_new_map(&mut self) -> CandleResult<()> {
        let reg = self.fetch_u8()? as usize;
        let id = self.get_heap().allocate(Object::Map(HashMap::new()));
        self.set_register(reg, Value::ObjectRef(id));
        self.maybe_run_gc();
        Ok(())
    }

    pub fn exec_map_len(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;

        let len = {
            let object = self.get_heap().get(ref_val)?;
            match object {
                Object::Map(vec) => vec.len(),
                _ => {
                    return Err(CandleError::ObjectTypeMismatch {
                        expected: Cow::Borrowed(&[ObjectType::Map]),
                        found: object.object_type(),
                    });
                }
            }
        };

        self.set_register(target_reg, Value::from_f64(len as f64)?);
        Ok(())
    }

    pub fn exec_map_contains(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;
        let key_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;
        let key_val = self.get_register(key_reg);

        let contains = {
            let object = self.get_heap().get(ref_val)?;
            match object {
                Object::Map(map) => map.contains_key(&key_val),
                _ => {
                    return Err(CandleError::ObjectTypeMismatch {
                        expected: Cow::Borrowed(&[ObjectType::Map]),
                        found: object.object_type(),
                    });
                }
            }
        };

        self.set_register(target_reg, Value::Bool(contains));
        Ok(())
    }

    pub fn exec_map_get(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;
        let key_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;
        let key_val = self.get_register(key_reg);

        let val = {
            let object = self.get_heap().get(ref_val)?;

            match object {
                Object::Map(map) => map
                    .get(&key_val)
                    .cloned()
                    .ok_or(CandleError::MapKeyNotFound(key_val))?,
                _ => {
                    return Err(CandleError::ObjectTypeMismatch {
                        expected: Cow::Borrowed(&[ObjectType::Map]),
                        found: object.object_type(),
                    });
                }
            }
        };

        self.set_register(target_reg, val);
        Ok(())
    }

    pub fn exec_map_set(&mut self) -> CandleResult<()> {
        let ref_reg = self.fetch_u8()? as usize;
        let key_reg = self.fetch_u8()? as usize;
        let value_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;
        let key_val = self.get_register(key_reg);
        let value = self.get_register(value_reg);

        let object = self.get_heap().get_mut(ref_val)?;
        match object {
            Object::Map(map) => {
                map.insert(key_val, value);
                Ok(())
            }
            _ => Err(CandleError::ObjectTypeMismatch {
                expected: Cow::Borrowed(&[ObjectType::Map]),
                found: object.object_type(),
            }),
        }
    }

    pub fn exec_map_remove(&mut self) -> CandleResult<()> {
        let ref_reg = self.fetch_u8()? as usize;
        let key_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;
        let key_val = self.get_register(key_reg);

        let object = self.get_heap().get_mut(ref_val)?;
        match object {
            Object::Map(map) => {
                map.remove(&key_val)
                    .ok_or(CandleError::MapKeyNotFound(key_val))?;
                Ok(())
            }
            _ => Err(CandleError::ObjectTypeMismatch {
                expected: Cow::Borrowed(&[ObjectType::Map]),
                found: object.object_type(),
            }),
        }
    }
}
