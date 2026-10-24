pub const CANDLE_VERSION: u16 = 1;

pub const VM_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const BYTECODE_MAGIC: &[u8; 4] = b"CAND";

pub const MODULE_SPACE_SIZE: u16 = (u16::MAX / 2) + 1;
pub const PLUGIN_MODULE_BASE: u16 = MODULE_SPACE_SIZE;

pub const MAX_FUNCTIONS: u16 = u16::MAX;
pub const MAX_FUNCTION_PARAMETERS: u8 = 8;
pub const MAX_CAPTURE_COUNT: u8 = 8;

pub const MAX_RESOURCES: u16 = u16::MAX;
pub const MAX_RESOURCE_SIZE: u32 = 64 * 1024 * 1024; // 64 MiB

pub const MAX_CALL_STACK: usize = 1024;
pub const MAX_STACKTRACE_FRAMES: usize = 10;

pub const ALLOCATIONS_BEFORE_GC: usize = 1024;
