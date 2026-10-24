use crate::error::{CandleError, CandleResult};
use crate::heap::{Object, ObjectType};
use crate::value::{Value, ValueType};
use crate::vm::Vm;
use std::borrow::Cow;


fn parse_f64(input: &str) -> CandleResult<f64> {
    let input = input.trim();
    let (negative, unsigned) = if let Some(value) = input.strip_prefix('-') {
        (true, value)
    } else if let Some(value) = input.strip_prefix('+') {
        (false, value)
    } else {
        (false, input)
    };

    if let Some(hex) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        match u64::from_str_radix(hex, 16) {
            Ok(value) => {
                let value = value as f64;
                Ok(if negative { -value } else { value })
            }
            Err(_) => Err(CandleError::InvalidNumber),
        }
    } else {
        input.parse::<f64>().map_err(|_| {
            CandleError::InvalidNumber
        })
    }
}

impl Vm {
    pub fn exec_new_string(&mut self) -> CandleResult<()> {
        let reg = self.fetch_u8()? as usize;
        let string = self.fetch_string()?;
        let id = self.get_heap().allocate(Object::String(string));
        self.set_register(reg, Value::ObjectRef(id));
        self.maybe_run_gc();
        Ok(())
    }

    pub fn exec_string_len(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;

        let len = {
            let object = self.get_heap().get(ref_val)?;
            match object {
                Object::String(string) => string.chars().count(),
                _ => {
                    return Err(CandleError::ObjectTypeMismatch {
                        expected: Cow::Borrowed(&[ObjectType::String]),
                        found: object.object_type(),
                    });
                }
            }
        };

        self.set_register(target_reg, Value::from_f64(len as f64)?);
        Ok(())
    }

    pub fn exec_string_concat(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let a_reg = self.fetch_u8()? as usize;
        let b_reg = self.fetch_u8()? as usize;

        let a_value = self.get_register(a_reg);
        let b_value = self.get_register(b_reg);

        let a_string = match a_value {
            Value::ObjectRef(id) => {
                let object = self.get_heap().get(id)?;
                match object {
                    Object::String(string) => string.clone(),
                    _ => {
                        return Err(CandleError::ObjectTypeMismatch {
                            expected: Cow::Borrowed(&[ObjectType::String]),
                            found: object.object_type(),
                        });
                    }
                }
            }
            _ => {
                return Err(CandleError::ValueTypeMismatch {
                    expected: Cow::Borrowed(&[ValueType::ObjectRef]),
                    found: a_value.value_type(),
                });
            }
        };

        let b_string = match b_value {
            Value::ObjectRef(id) => {
                let object = self.get_heap().get(id)?;
                match object {
                    Object::String(string) => string.clone(),
                    _ => {
                        return Err(CandleError::ObjectTypeMismatch {
                            expected: Cow::Borrowed(&[ObjectType::String]),
                            found: object.object_type(),
                        });
                    }
                }
            }
            _ => {
                return Err(CandleError::ValueTypeMismatch {
                    expected: Cow::Borrowed(&[ValueType::ObjectRef]),
                    found: b_value.value_type(),
                });
            }
        };

        let mut result = String::with_capacity(
            a_string.len() + b_string.len()
        );
        result.push_str(&a_string);
        result.push_str(&b_string);

        let id = self.get_heap().allocate(Object::String(result));

        self.set_register(target_reg, Value::ObjectRef(id));
        Ok(())
    }

    pub fn exec_string_from_chars(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;

        let object = self.get_heap().get(ref_val)?;
        match object {
            Object::Array(vec) => {
                let mut string = String::with_capacity(vec.len());

                for val in vec {
                    match val {
                        Value::Char(c) => string.push(*c),
                        _ => {
                            return Err(CandleError::ArrayTypeMismatch {
                                expected: ValueType::Char,
                                found: val.value_type(),
                            });
                        }
                    }
                }

                let id = self.get_heap().allocate(Object::String(string));

                self.set_register(target_reg, Value::ObjectRef(id));
                Ok(())
            }
            _ => Err(CandleError::ObjectTypeMismatch {
                expected: Cow::Borrowed(&[ObjectType::Array]),
                found: object.object_type(),
            }),
        }
    }

    pub fn exec_string_to_chars(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;

        let object = self.get_heap().get(ref_val)?;
        match object {
            Object::String(str) => {
                let chars: Vec<Value> = str.chars().map(Value::Char).collect();

                let id = self.get_heap().allocate(Object::Array(chars));

                self.set_register(target_reg, Value::ObjectRef(id));
                Ok(())
            }
            _ => Err(CandleError::ObjectTypeMismatch {
                expected: Cow::Borrowed(&[ObjectType::String]),
                found: object.object_type(),
            }),
        }
    }

    pub fn exec_string_from_bytes(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;

        let object = self.get_heap().get(ref_val)?;
        match object {
            Object::Buffer(bytes) => {
                let string = String::from_utf8(bytes.clone()).map_err(|_| CandleError::InvalidString)?;
                let id = self.get_heap().allocate(Object::String(string));
                self.set_register(target_reg, Value::ObjectRef(id));
                Ok(())
            }
            _ => Err(CandleError::ObjectTypeMismatch {
                expected: Cow::Borrowed(&[ObjectType::Buffer]),
                found: object.object_type(),
            }),
        }
    }

    pub fn exec_string_to_bytes(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let ref_reg = self.fetch_u8()? as usize;

        let ref_val = self.get_register(ref_reg).as_object_ref()?;

        let heap = self.get_heap();
        let object = heap.get(ref_val)?;
        match object {
            Object::String(str) => {
                let bytes = str.as_bytes();
                let id = heap.allocate(Object::Buffer(bytes.to_vec()));
                self.set_register(target_reg, Value::ObjectRef(id));
                Ok(())
            }
            _ => Err(CandleError::ObjectTypeMismatch {
                expected: Cow::Borrowed(&[ObjectType::String]),
                found: object.object_type(),
            }),
        }
    }

    pub fn exec_string_of(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let value_reg = self.fetch_u8()? as usize;
        let value = self.get_register(value_reg);

        let string = match value {
            Value::ObjectRef(id) => format!("{}", self.get_heap().get(id)?),
            _ => format!("{}", value),
        };

        let id = self.get_heap().allocate(Object::String(string));
        self.set_register(target_reg, Value::ObjectRef(id));
        self.maybe_run_gc();
        Ok(())
    }

    pub fn exec_string_lowercase(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let string_reg = self.fetch_u8()? as usize;
        let string = self.get_string(string_reg)?;
        let id = self.get_heap().allocate(Object::String(string.to_lowercase()));
        self.set_register(target_reg, Value::ObjectRef(id));
        self.maybe_run_gc();
        Ok(())
    }

    pub fn exec_string_uppercase(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let string_reg = self.fetch_u8()? as usize;
        let string = self.get_string(string_reg)?;
        let id = self.get_heap().allocate(Object::String(string.to_uppercase()));
        self.set_register(target_reg, Value::ObjectRef(id));
        self.maybe_run_gc();
        Ok(())
    }

    pub fn exec_string_to_number(&mut self) -> CandleResult<()> {
        let target_reg = self.fetch_u8()? as usize;
        let string_reg = self.fetch_u8()? as usize;
        let string = self.get_string(string_reg)?;
        let number = parse_f64(&string)?;
        self.set_register(target_reg, Value::from_f64(number)?);
        Ok(())
    }

    fn get_string(&mut self, reg: usize) -> CandleResult<String> {
        let object_ref = self.get_register(reg).as_object_ref()?;
        let object = self.get_heap().get(object_ref)?;
        match object {
            Object::String(string) => Ok(string.clone()),
            _ => Err(CandleError::ObjectTypeMismatch {
                expected: Cow::Borrowed(&[ObjectType::String]),
                found: object.object_type(),
            }),
        }
    }
}
