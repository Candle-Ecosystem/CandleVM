use crate::constants::{CANDLE_VERSION, VM_VERSION};
use crate::error::CandleResult;
use crate::value::Value;
use crate::vm::Vm;
use libloading::{Library, Symbol};
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

type CandleVersion = extern "Rust" fn() -> u16;
type CandlePluginInit = extern "Rust" fn() -> Plugin;

pub struct Plugin {
    name: String,
    version: String,
    modules: Vec<PluginModule>,
}

impl Plugin {
    pub fn new(name: String, version: String, modules: Vec<PluginModule>) -> Plugin {
        Self {
            name,
            version,
            modules,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_version(&self) -> &String {
        &self.version
    }

    pub fn get_modules(&self) -> &Vec<PluginModule> {
        &self.modules
    }
}

pub struct PluginModule {
    name: String,
    functions: Vec<PluginFunction>,
}

impl PluginModule {
    pub fn new(name: String, functions: Vec<PluginFunction>) -> Self {
        Self { name, functions }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_functions(&self) -> &Vec<PluginFunction> {
        &self.functions
    }
}

pub struct PluginFunction {
    name: String,
    parameter_count: u8,
    handler: Arc<dyn Fn(&mut Vm, &[Value]) -> CandleResult<Value>>,
}

impl PluginFunction {
    pub fn new(
        name: String,
        parameter_count: u8,
        handler: Arc<dyn Fn(&mut Vm, &[Value]) -> CandleResult<Value>>,
    ) -> Self {
        Self {
            name,
            parameter_count,
            handler,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_parameter_count(&self) -> u8 {
        self.parameter_count
    }

    pub fn get_handler(&self) -> Arc<dyn Fn(&mut Vm, &[Value]) -> CandleResult<Value>> {
        Arc::clone(&self.handler)
    }
}

pub struct LoadedPlugin {
    pub library: Library,
    pub plugin: Plugin,
}

pub fn load_plugin(path: PathBuf) -> Result<LoadedPlugin, Box<dyn Error>> {
    let library = unsafe { Library::new(path)? };

    let version_fn: Symbol<CandleVersion> = unsafe { library.get(b"candle_version")? };

    let version = version_fn();
    if version != CANDLE_VERSION {
        return Err(format!(
            "Incompatible Candle version (current={}, plugin={})",
            VM_VERSION, version
        )
            .into());
    }

    let init_fn: Symbol<CandlePluginInit> = unsafe { library.get(b"candle_plugin_init")? };
    let plugin = init_fn();

    Ok(LoadedPlugin { library, plugin })
}
