use crate::heap::ObjectType;
use crate::value::{Value, ValueType};
use std::borrow::Cow;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug)]
pub enum CandleError {
    Custom(String),

    // Bytecode / execution
    InvalidOpcode(u8),
    InvalidValueType(u8),
    UnexpectedBytecodeEnd,
    InvalidJump(usize),

    // Types / values
    ValueTypeMismatch {
        expected: Cow<'static, [ValueType]>,
        found: ValueType,
    },
    ObjectTypeMismatch {
        expected: Cow<'static, [ObjectType]>,
        found: ObjectType,
    },
    ArrayTypeMismatch {
        expected: ValueType,
        found: ValueType,
    },
    InvalidNumber,
    InvalidChar,
    ModuleName(String),
    FunctionName {
        module: String,
        name: String,
    },
    InvalidModuleRef(u16),
    InvalidFunctionRef {
        module_id: u16,
        id: u16,
    },
    InvalidHeapRef(u32),
    InvalidString,
    InvalidInt(f64),
    ByteLimit(f64),
    UsizeLimit(f64),
    ValueNotSerializable(ValueType),

    // Runtime operations
    DivisionByZero,
    ArrayIndexOutOfBounds {
        index: usize,
        len: usize,
    },
    BufferIndexOutOfBounds {
        index: usize,
        len: usize,
    },
    MapKeyNotFound(Value),
    ResourceNotFound(String),
    ReadOnlyGlobal(String),
    Os(Option<String>),
    InvalidInput(String),

    // Call stack
    CallstackOverflow,
    CallstackUnderflow,
}

impl CandleError {
    pub fn get_type(&self) -> &'static str {
        match self {
            Self::Custom(_) => "Custom",

            Self::InvalidOpcode(_) => "InvalidOpcode",
            Self::InvalidValueType(_) => "InvalidValueType",
            Self::UnexpectedBytecodeEnd => "UnexpectedBytecodeEnd",
            Self::InvalidJump(_) => "InvalidJump",

            Self::ValueTypeMismatch { .. } => "ValueTypeMismatch",
            Self::ObjectTypeMismatch { .. } => "ObjectTypeMismatch",
            Self::ArrayTypeMismatch { .. } => "ArrayTypeMismatch",
            Self::InvalidNumber => "InvalidNumber",
            Self::InvalidChar => "InvalidChar",
            Self::ModuleName(_) => "ModuleName",
            Self::FunctionName { .. } => "FunctionName",
            Self::InvalidModuleRef(_) => "InvalidModuleRef",
            Self::InvalidFunctionRef { .. } => "InvalidFunctionRef",
            Self::InvalidHeapRef(_) => "InvalidHeapRef",
            Self::InvalidString => "InvalidString",
            Self::InvalidInt(_) => "InvalidInt",
            Self::ByteLimit(_) => "ByteLimit",
            Self::UsizeLimit(_) => "UsizeLimit",
            Self::ValueNotSerializable(_) => "ValueNotSerializable",

            Self::DivisionByZero => "DivisionByZero",
            Self::ArrayIndexOutOfBounds { .. } => "ArrayIndexOutOfBounds",
            Self::BufferIndexOutOfBounds { .. } => "BufferIndexOutOfBounds",
            Self::MapKeyNotFound(_) => "MapKeyNotFound",
            Self::ResourceNotFound(_) => "ResourceNotFound",
            Self::ReadOnlyGlobal(_) => "ReadOnlyGlobal",
            Self::Os(_) => "Os",
            Self::InvalidInput(_) => "InvalidInput",

            Self::CallstackOverflow => "CallstackOverflow",
            Self::CallstackUnderflow => "CallstackUnderflow",
        }
    }

    pub fn get_message(&self) -> String {
        match self {
            Self::Custom(msg) => msg.clone(),

            Self::InvalidOpcode(opcode) => format!("Invalid opcode found (0x{:02X})", opcode),

            Self::InvalidValueType(opcode) => {
                format!("Invalid value type found (0x{:02X})", opcode)
            }

            Self::UnexpectedBytecodeEnd => "Reached end of bytecode while reading".into(),

            Self::InvalidJump(pos) => format!("Invalid jump target ({})", pos),

            Self::ValueTypeMismatch { expected, found } => {
                let expected = match expected.as_ref() {
                    [] => "nothing".into(),
                    [ty] => format!("{:?}", ty),
                    _ => expected
                        .iter()
                        .map(|ty| format!("{:?}", ty))
                        .collect::<Vec<_>>()
                        .join(" or "),
                };

                format!("Expected {} value, but found {:?} value", expected, found)
            }

            Self::ObjectTypeMismatch { expected, found } => {
                let expected = match expected.as_ref() {
                    [] => "nothing".into(),
                    [ty] => format!("{:?}", ty),
                    _ => expected
                        .iter()
                        .map(|ty| format!("{:?}", ty))
                        .collect::<Vec<_>>()
                        .join(" or "),
                };

                format!(
                    "Expected {} object, but found {:?} object",
                    expected, found
                )
            }

            Self::ArrayTypeMismatch { expected, found } => format!(
                "Expected Array of {:?} values, but found an Array of {:?} values",
                expected, found
            ),

            Self::InvalidNumber => "Invalid Number".into(),

            Self::InvalidChar => "Invalid UTF-8 character".into(),

            Self::ModuleName(name) => format!("Module not found (module=\"{}\")", name),

            Self::FunctionName { module, name } => {
                format!(
                    "Function not found (module=\"{}\", function=\"{}\")",
                    name, module
                )
            }

            Self::InvalidModuleRef(id) => format!("Invalid ModuleRef (id={})", id),

            Self::InvalidFunctionRef { module_id, id } => {
                format!("Invalid FunctionRef (module_id={}, id={}", module_id, id)
            }

            Self::InvalidHeapRef(id) => format!("Invalid HeapRef (id={}", id),

            Self::InvalidString => "Invalid UTF-8 string".into(),

            Self::InvalidInt(value) => {
                format!("Number must be a finite integer ({})", value)
            }

            Self::ByteLimit(value) => {
                format!("Number does not fit into a byte (0..255) ({})", value)
            }

            Self::UsizeLimit(value) => format!("Number does not fit into usize ({})", value),

            Self::ValueNotSerializable(value_type) => {
                format!("{:?} values cannot be serialized", value_type)
            }

            Self::DivisionByZero => "Attempted to divide a Number by zero".into(),

            Self::ArrayIndexOutOfBounds { index, len } => format!(
                "Attempted to access an Array index that is out of bounds (requested_index={}, length={})",
                index, len
            ),

            Self::BufferIndexOutOfBounds { index, len } => format!(
                "Attempted to access a Buffer index that is out of bounds (requested_index={}, length={})",
                index, len
            ),

            Self::MapKeyNotFound(key) => format!(
                "Attempted to access a Map key that was not found (key={})",
                key
            ),

            Self::ResourceNotFound(id) => format!(
                "Attempted to access a resource that was not found (id={})",
                id
            ),

            Self::ReadOnlyGlobal(global) => format!("Attempted to modify a read-only global ({})", global),

            Self::Os(Some(message)) => message.clone(),
            Self::Os(None) => "An OS error occurred".into(),

            Self::InvalidInput(msg) => format!("{}", msg),

            Self::CallstackOverflow => "Callstack is full".into(),

            Self::CallstackUnderflow => "Callstack is empty".into(),
        }
    }
}

impl Display for CandleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} Error: {}", self.get_type(), self.get_message())
    }
}

impl PartialEq for CandleError {
    fn eq(&self, other: &Self) -> bool {
        if self.get_type() != other.get_type() {
            return false;
        }

        match (self, other) {
            (Self::Custom(a), Self::Custom(b)) => a == b,
            (Self::InvalidInput(a), Self::InvalidInput(b)) => a == b,
            (Self::Os(a), Self::Os(b)) => a == b,
            _ => true,
        }
    }
}

impl Eq for CandleError {}

pub type CandleResult<T> = Result<T, CandleError>;
