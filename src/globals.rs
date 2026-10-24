use crate::value::Value;

pub struct Global {
    pub value: Value,
    pub read_only: bool,
}