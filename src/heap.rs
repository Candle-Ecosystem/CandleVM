use crate::error::{CandleError, CandleResult};
use crate::value::Value;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum ObjectType {
    Array = 0,
    Map = 1,
    String = 2,
    Error = 3,
    Buffer = 4,
    Closure = 5,
    Native = 6,
}

impl TryFrom<u8> for ObjectType {
    type Error = u8;

    fn try_from(object: u8) -> Result<Self, <ObjectType as TryFrom<u8>>::Error> {
        match object {
            0 => Ok(ObjectType::Array),
            1 => Ok(ObjectType::Map),
            2 => Ok(ObjectType::String),
            3 => Ok(ObjectType::Error),
            4 => Ok(ObjectType::Buffer),
            5 => Ok(ObjectType::Closure),
            6 => Ok(ObjectType::Native),
            _ => Err(object),
        }
    }
}

pub enum Object {
    Array(Vec<Value>),
    Map(HashMap<Value, Value>),
    String(String),
    Error(CandleError),
    Buffer(Vec<u8>),
    Closure(u16, u16, Vec<Value>),
    Native(Box<dyn NativeObject>),
}

impl Object {
    pub fn object_type(&self) -> ObjectType {
        match self {
            Object::Array(_) => ObjectType::Array,
            Object::Map(_) => ObjectType::Map,
            Object::String(_) => ObjectType::String,
            Object::Error(_) => ObjectType::Error,
            Object::Buffer(_) => ObjectType::Buffer,
            Object::Closure(_, _, _) => ObjectType::Closure,
            Object::Native(_) => ObjectType::Native,
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Array(arr) => write!(f, "Array {{ length={} }}", arr.len()),
            Object::Map(map) => write!(f, "Map {{ length={} }}", map.len()),
            Object::String(str) => write!(f, "{}", str),
            Object::Error(err) => write!(
                f,
                "Error {{ type={}, message=\"{}\" }}",
                err.get_type(),
                err.get_message()
            ),
            Object::Buffer(buffer) => write!(f, "Buffer {{ length={} }}", buffer.len()),
            Object::Closure(module_id, function_id, captures) => write!(
                f,
                "Closure {{ module_id={}, function_id={}, captures={} }}",
                module_id,
                function_id,
                captures.len()
            ),
            Object::Native(obj) => write!(f, "{}", obj),
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Array(a), Object::Array(b)) => a == b,
            (Object::Map(a), Object::Map(b)) => a == b,
            (Object::String(a), Object::String(b)) => a == b,
            (Object::Error(a), Object::Error(b)) => a == b,
            (Object::Buffer(a), Object::Buffer(b)) => a == b,
            (
                Object::Closure(a_module, a_func, a_captures),
                Object::Closure(b_module, b_func, b_captures),
            ) => a_module == b_module && a_func == b_func && a_captures == b_captures,
            (Object::Native(a), Object::Native(b)) => a.native_eq(b.as_ref()),
            _ => false,
        }
    }
}

pub trait NativeObject: Any + Display {
    fn as_any(&self) -> &dyn Any;

    fn native_eq(&self, other: &dyn NativeObject) -> bool;
}

pub struct Heap {
    objects: HashMap<u32, Object>,
    next_id: u32,
    free_ids: Vec<u32>,
    allocations_since_gc: usize,
    debug: bool,
}

impl Heap {
    pub fn new(debug: bool) -> Self {
        Self {
            objects: HashMap::new(),
            next_id: 0,
            free_ids: Vec::new(),
            allocations_since_gc: 0,
            debug,
        }
    }

    pub fn ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.objects.keys().copied()
    }

    pub fn allocate(&mut self, object: Object) -> u32 {
        let id = match self.free_ids.pop() {
            Some(id) => id,
            None => {
                let id = self.next_id;
                self.next_id += 1;
                id
            }
        };

        self.objects.insert(id, object);

        if self.debug {
            println!("(HEAP) DEBUG: Allocated object #{}", id);
        }

        self.allocations_since_gc += 1;
        id
    }

    pub fn get(&self, id: u32) -> CandleResult<&Object> {
        self.objects.get(&id).ok_or(CandleError::InvalidHeapRef(id))
    }

    pub fn get_mut(&mut self, id: u32) -> CandleResult<&mut Object> {
        self.objects
            .get_mut(&id)
            .ok_or(CandleError::InvalidHeapRef(id))
    }

    pub fn remove(&mut self, id: u32) {
        if let Some(object) = self.objects.remove(&id) {
            drop(object);
            self.free_ids.push(id);
        }
    }

    pub fn get_allocations_since_gc(&self) -> usize {
        self.allocations_since_gc
    }

    pub fn set_allocations_since_gc(&mut self, size: usize) {
        self.allocations_since_gc = size;
    }
}
