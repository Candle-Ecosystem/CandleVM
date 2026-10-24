use crate::error::{CandleError, CandleResult};
use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Sub};

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ValueType {
    Null = 0,
    Byte = 1,
    Number = 2,
    Bool = 3,
    Char = 4,
    ModuleRef = 5,
    FunctionRef = 6,
    ObjectRef = 7,
}

impl TryFrom<u8> for ValueType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ValueType::Null),
            1 => Ok(ValueType::Byte),
            2 => Ok(ValueType::Number),
            3 => Ok(ValueType::Bool),
            4 => Ok(ValueType::Char),
            5 => Ok(ValueType::ModuleRef),
            6 => Ok(ValueType::FunctionRef),
            7 => Ok(ValueType::ObjectRef),
            _ => Err(value),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Value {
    Null,
    Byte(u8),
    Number(f64),
    Bool(bool),
    Char(char),
    ModuleRef(u16),
    FunctionRef(u16, u16),
    ObjectRef(u32),
}

impl Value {
    pub fn serialize(&self, bytecode: &mut Vec<u8>) {
        match self {
            Value::Null => {
                bytecode.push(ValueType::Null as u8);
            }
            Value::Byte(v) => {
                bytecode.push(ValueType::Byte as u8);
                bytecode.push(*v);
            }
            Value::Number(v) => {
                bytecode.push(ValueType::Number as u8);
                bytecode.extend_from_slice(&v.to_le_bytes());
            }
            Value::Bool(v) => {
                bytecode.push(ValueType::Bool as u8);
                bytecode.push(*v as u8);
            }
            Value::Char(v) => {
                bytecode.push(ValueType::Char as u8);
                bytecode.extend_from_slice(&(*v as u32).to_le_bytes());
            }
            Value::ModuleRef(v) => {
                bytecode.push(ValueType::ModuleRef as u8);
                bytecode.extend_from_slice(&v.to_le_bytes());
            }
            Value::FunctionRef(v, w) => {
                bytecode.push(ValueType::FunctionRef as u8);
                bytecode.extend_from_slice(&v.to_le_bytes());
                bytecode.extend_from_slice(&w.to_le_bytes());
            }
            Value::ObjectRef(v) => {
                bytecode.push(ValueType::ObjectRef as u8);
                bytecode.extend_from_slice(&v.to_le_bytes());
            }
        }
    }

    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Null => ValueType::Null,
            Value::Byte(_) => ValueType::Byte,
            Value::Number(_) => ValueType::Number,
            Value::Bool(_) => ValueType::Bool,
            Value::Char(_) => ValueType::Char,
            Value::ModuleRef(_) => ValueType::ModuleRef,
            Value::FunctionRef(_, _) => ValueType::FunctionRef,
            Value::ObjectRef(_) => ValueType::ObjectRef,
        }
    }

    pub fn from_f64(value: f64) -> CandleResult<Value> {
        if value.is_nan() | !value.is_finite() {
            return Err(CandleError::InvalidNumber);
        }

        if value.fract() == 0.0
            && (0.0..=u8::MAX as f64).contains(&value)
        {
            Ok(Value::Byte(value as u8))
        } else {
            Ok(Value::Number(value))
        }
    }

    pub fn as_f64(&self) -> CandleResult<f64> {
        match self {
            Value::Number(v) => Ok(*v),
            Value::Byte(v) => Ok(*v as f64),
            _ => Err(CandleError::ValueTypeMismatch {
                expected: Cow::Borrowed(&[ValueType::Number, ValueType::Byte]),
                found: self.value_type(),
            }),
        }
    }

    pub fn as_u8(&self) -> CandleResult<u8> {
        match self {
            Value::Byte(v) => Ok(*v),
            Value::Number(v) => {
                let v = *v;

                if v.fract() != 0.0 {
                    Err(CandleError::InvalidInt(v))
                } else if !(0.0..=u8::MAX as f64).contains(&v) {
                    Err(CandleError::ByteLimit(v))
                } else {
                    Ok(v as u8)
                }
            }
            _ => Err(CandleError::ValueTypeMismatch {
                expected: Cow::Borrowed(&[ValueType::Number, ValueType::Byte]),
                found: self.value_type(),
            }),
        }
    }

    pub fn as_usize(&self) -> CandleResult<usize> {
        match self {
            Value::Number(v) => {
                let v = *v;
                if v < 0.0 || v.fract() != 0.0 {
                    Err(CandleError::InvalidInt(v))
                } else if v > usize::MAX as f64 {
                    Err(CandleError::UsizeLimit(v))
                } else {
                    Ok(v as usize)
                }
            }
            Value::Byte(v) => Ok(*v as usize),
            _ => Err(CandleError::ValueTypeMismatch {
                expected: Cow::Borrowed(&[ValueType::Number, ValueType::Byte]),
                found: self.value_type(),
            }),
        }
    }

    pub fn as_bool(&self) -> CandleResult<bool> {
        match self {
            Value::Bool(v) => Ok(*v),
            _ => Err(CandleError::ValueTypeMismatch {
                expected: Cow::Borrowed(&[ValueType::Bool]),
                found: self.value_type(),
            }),
        }
    }

    pub fn as_char(&self) -> CandleResult<char> {
        match self {
            Value::Char(v) => Ok(*v),
            _ => Err(CandleError::ValueTypeMismatch {
                expected: Cow::Borrowed(&[ValueType::Char]),
                found: self.value_type(),
            }),
        }
    }

    pub fn as_module_ref(&self) -> CandleResult<u16> {
        match self {
            Value::ModuleRef(v) => Ok(*v),
            _ => Err(CandleError::ValueTypeMismatch {
                expected: Cow::Borrowed(&[ValueType::ModuleRef]),
                found: self.value_type(),
            }),
        }
    }

    pub fn as_function_ref(&self) -> CandleResult<(u16, u16)> {
        match self {
            Value::FunctionRef(v, w) => Ok((*v, *w)),
            _ => Err(CandleError::ValueTypeMismatch {
                expected: Cow::Borrowed(&[ValueType::FunctionRef]),
                found: self.value_type(),
            }),
        }
    }

    pub fn as_object_ref(&self) -> CandleResult<u32> {
        match self {
            Value::ObjectRef(v) => Ok(*v),
            _ => Err(CandleError::ValueTypeMismatch {
                expected: Cow::Borrowed(&[ValueType::ObjectRef]),
                found: self.value_type(),
            }),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Byte(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Char(v) => write!(f, "{}", v),
            Value::ModuleRef(v) => write!(f, "ModuleRef {{ id={} }}", u32::from(*v)),
            Value::FunctionRef(v, w) => {
                write!(f, "FunctionRef {{ module_id={}, id={} }}", u32::from(*v), w)
            }
            Value::ObjectRef(v) => write!(f, "ObjectRef {{ id={} }}", v),
        }
    }
}


impl Add for Value {
    type Output = CandleResult<Value>;

    fn add(self, rhs: Value) -> CandleResult<Value> {
        Ok(Value::from_f64(self.as_f64()? + rhs.as_f64()?)?)
    }
}

impl Sub for Value {
    type Output = CandleResult<Value>;

    fn sub(self, rhs: Value) -> CandleResult<Value> {
        Ok(Value::from_f64(self.as_f64()? - rhs.as_f64()?)?)
    }
}

impl Mul for Value {
    type Output = CandleResult<Value>;

    fn mul(self, rhs: Value) -> CandleResult<Value> {
        Ok(Value::from_f64(self.as_f64()? * rhs.as_f64()?)?)
    }
}

impl Div for Value {
    type Output = CandleResult<Value>;

    fn div(self, rhs: Value) -> CandleResult<Value> {
        let a = self.as_f64()?;
        let b = rhs.as_f64()?;

        if b == 0.0 {
            return Err(CandleError::DivisionByZero);
        }

        Ok(Value::from_f64(a / b)?)
    }
}

impl Rem for Value {
    type Output = CandleResult<Value>;

    fn rem(self, rhs: Value) -> CandleResult<Value> {
        let a = self.as_f64()?;
        let b = rhs.as_f64()?;

        if b == 0.0 {
            return Err(CandleError::DivisionByZero);
        }

        Ok(Value::from_f64(a % b)?)
    }
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut vec: Vec<u8> = Vec::new();
        self.serialize(&mut vec).hash(state);
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,

            (Value::Byte(_), Value::Byte(_))
            | (Value::Byte(_), Value::Number(_))
            | (Value::Number(_), Value::Byte(_))
            | (Value::Number(_), Value::Number(_)) => {
                matches!(
                    (self.as_f64(), other.as_f64()),
                    (Ok(a), Ok(b)) if a == b
                )
            }

            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Char(a), Value::Char(b)) => a == b,
            (Value::ObjectRef(a), Value::ObjectRef(b)) => a == b,

            _ => false,
        }
    }
}

impl Eq for Value {}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Byte(_), Value::Byte(_))
            | (Value::Byte(_), Value::Number(_))
            | (Value::Number(_), Value::Byte(_))
            | (Value::Number(_), Value::Number(_)) => {
                self.as_f64().ok()?.partial_cmp(&other.as_f64().ok()?)
            }

            (Value::Bool(a), Value::Bool(b)) => a.partial_cmp(b),

            (Value::Null, Value::Null) => Some(Ordering::Equal),

            _ => None,
        }
    }
}

impl BitAnd for Value {
    type Output = CandleResult<Value>;

    fn bitand(self, rhs: Value) -> CandleResult<Value> {
        let a = self.as_bool()?;
        let b = rhs.as_bool()?;
        Ok(Value::Bool(a & b))
    }
}

impl BitOr for Value {
    type Output = CandleResult<Value>;

    fn bitor(self, rhs: Value) -> CandleResult<Value> {
        let a = self.as_bool()?;
        let b = rhs.as_bool()?;
        Ok(Value::Bool(a | b))
    }
}

impl BitXor for Value {
    type Output = CandleResult<Value>;

    fn bitxor(self, rhs: Self) -> CandleResult<Value> {
        let a = self.as_bool()?;
        let b = rhs.as_bool()?;
        Ok(Value::Bool(a ^ b))
    }
}

impl Not for Value {
    type Output = CandleResult<Value>;

    fn not(self) -> CandleResult<Value> {
        let bool = self.as_bool()?;
        Ok(Value::Bool(!bool))
    }
}
