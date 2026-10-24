use crate::constants::{
    BYTECODE_MAGIC, CANDLE_VERSION, MAX_CAPTURE_COUNT, MAX_FUNCTIONS, MAX_FUNCTION_PARAMETERS,
    MAX_RESOURCES, MAX_RESOURCE_SIZE, MODULE_SPACE_SIZE, PLUGIN_MODULE_BASE,
};
use crate::error::CandleResult;
use crate::function::Function;
use crate::module::Module;
use crate::plugin::{load_plugin, LoadedPlugin};
use crate::value::Value;
use crate::vm::Vm;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::Arc;

pub struct LoadedProgram {
    pub modules: HashMap<u16, Module>,
    pub plugins: Vec<LoadedPlugin>,
    pub resources: HashMap<String, Vec<u8>>,
    pub bytecode: Vec<u8>,
}

#[derive(Debug)]
pub enum LoaderError {
    InvalidMagic {
        expected: [u8; 4],
        found: [u8; 4],
    },
    UnsupportedVersion {
        expected: u16,
        found: u16,
    },
    InvalidString,
    ModuleNameAlreadyUsed(String),
    FunctionNameAlreadyUsed {
        module_name: String,
        name: String,
    },
    ReachedModuleLimit,
    ReachedPluginModuleLimit,
    ReachedFunctionLimit,
    ReachedFunctionParameterLimit,
    ReachedFunctionCaptureLimit,
    ReachedResourceLimit,
    ReachedResourceSizeLimit,
    ResourceIdAlreadyUsed(String),
    PluginNotFound(PathBuf),
    PluginNameAlreadyUsed(String),
    MissingPlugin {
        name: String,
        version: String,
    },
    PluginLoad {
        path: PathBuf,
        error: Box<dyn Error>,
    },
    UnexpectedEndOfFile,
}

impl LoaderError {
    pub fn get_type(&self) -> &'static str {
        match self {
            Self::InvalidMagic { .. } => "InvalidMagic",
            Self::UnsupportedVersion { .. } => "UnsupportedVersion",
            Self::InvalidString => "InvalidString",
            Self::ModuleNameAlreadyUsed(_) => "ModuleNameAlreadyUsed",
            Self::FunctionNameAlreadyUsed { .. } => "FunctionNameAlreadyUsed",
            Self::ReachedModuleLimit => "ReachedModuleLimit",
            Self::ReachedPluginModuleLimit => "ReachedPluginModuleLimit",
            Self::ReachedFunctionLimit => "ReachedFunctionLimit",
            Self::ReachedFunctionParameterLimit => "ReachedFunctionParameterLimit",
            Self::ReachedFunctionCaptureLimit => "ReachedFunctionCaptureLimit",
            Self::ReachedResourceLimit => "ReachedResourceLimit",
            Self::ReachedResourceSizeLimit => "ReachedResourceSizeLimit",
            Self::ResourceIdAlreadyUsed(_) => "ResourceIdAlreadyUsed",
            Self::PluginNameAlreadyUsed(_) => "PluginNameAlreadyUsed",
            Self::PluginNotFound(_) => "PluginNotFound",
            Self::MissingPlugin { .. } => "MissingPlugin",
            Self::PluginLoad { .. } => "PluginLoad",
            Self::UnexpectedEndOfFile => "UnexpectedEndOfFile",
        }
    }

    pub fn get_message(&self) -> String {
        match self {
            Self::InvalidMagic { expected, found } => format!(
                "Expected 0x{:08X}, found 0x{:08X}",
                u32::from_be_bytes(*expected),
                u32::from_be_bytes(*found)
            ),

            Self::UnsupportedVersion { expected, found } => {
                format!("Excepted Candle {}, found Candle {}", expected, found)
            }

            Self::InvalidString => "Read invalid string".into(),

            Self::ModuleNameAlreadyUsed(name) => {
                format!("The name \"{}\" is already used by another module", name)
            }

            Self::FunctionNameAlreadyUsed { module_name, name } => format!(
                "The name \"{}\" is already used by another function in module \"{}\"",
                name, module_name
            ),

            Self::ReachedModuleLimit => format!("Module count is limited to {}", MODULE_SPACE_SIZE),

            Self::ReachedPluginModuleLimit => {
                format!("Plugin module count is limited to {}", MODULE_SPACE_SIZE)
            }

            Self::ReachedFunctionLimit => {
                format!("Function count is limited to {} per module", MAX_FUNCTIONS)
            }

            Self::ReachedFunctionParameterLimit => format!(
                "Function parameter count is limited to {}",
                MAX_FUNCTION_PARAMETERS
            ),

            Self::ReachedFunctionCaptureLimit => {
                format!("Function capture count is limited to {}", MAX_CAPTURE_COUNT)
            }

            Self::ReachedResourceLimit => format!("Resource count is limited to {}", MAX_RESOURCES),

            Self::ReachedResourceSizeLimit => {
                format!("Resource size is limited to {} bytes", MAX_RESOURCE_SIZE)
            }

            Self::ResourceIdAlreadyUsed(id) => format!(
                "The id \"{}\" is already used by another resource", id
            ),

            Self::PluginNameAlreadyUsed(name) => format!(
                "The name \"{}\" is already used by another loaded plugin",
                name
            ),

            Self::PluginNotFound(path) => {
                format!("Could not find plugin at path: {}", path.display())
            }

            Self::MissingPlugin { name, version } => {
                format!(
                    "Program requires a plugin that could not be found or does not match the required version (name=\"{}\", version=\"{}\")",
                    name, version
                )
            }

            Self::PluginLoad { path, error } => format!(
                "Could not load plugin at path '{}':\n{}",
                path.display(),
                error
            ),

            Self::UnexpectedEndOfFile => "Reached end of file while reading".into(),
        }
    }
}

impl Display for LoaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} Error: {}", self.get_type(), self.get_message())
    }
}

pub type LoaderResult<T> = Result<T, LoaderError>;

pub struct Loader<'a> {
    data: &'a [u8],
    plugins: Vec<PathBuf>,
    offset: usize,
    debug: bool,
}

impl<'a> Loader<'a> {
    pub fn new(data: &'a [u8], plugins: Vec<PathBuf>, debug: bool) -> Self {
        Self {
            data,
            plugins,
            offset: 0,
            debug,
        }
    }

    pub fn load(mut self) -> LoaderResult<LoadedProgram> {
        self.read_header()?;

        let (mut modules, plugins) = self.load_plugins()?;
        self.check_plugins(&plugins)?;

        self.load_modules(&mut modules)?;

        let resources = self.load_resources()?;

        let bytecode = self.data[self.offset..].to_vec();

        Ok(LoadedProgram {
            modules,
            plugins,
            resources,
            bytecode,
        })
    }

    fn load_plugins(&self) -> LoaderResult<(HashMap<u16, Module>, Vec<LoadedPlugin>)> {
        if self.debug {
            println!("(LOADER) DEBUG: Loading plugins");
        }

        let mut modules = HashMap::new();
        let mut plugins = Vec::with_capacity(self.plugins.len());

        let mut plugin_id = 0;
        for path in &self.plugins {
            if self.debug {
                println!(
                    "(LOADER) DEBUG: Loading plugin #{} (path=\"{}\")",
                    plugin_id,
                    path.display()
                );
            }

            if !path.exists() || !path.is_file() {
                return Err(LoaderError::PluginNotFound(path.into()));
            }

            let plugin = load_plugin(path.into()).map_err(|error| LoaderError::PluginLoad {
                path: path.into(),
                error,
            })?;

            let plugin_name = plugin.plugin.get_name();

            if plugin.plugin.get_modules().len() > MODULE_SPACE_SIZE as usize {
                return Err(LoaderError::ReachedPluginModuleLimit);
            }

            if plugins
                .iter()
                .any(|loaded: &LoadedPlugin| loaded.plugin.get_name() == plugin_name)
            {
                return Err(LoaderError::PluginNameAlreadyUsed(plugin_name.into()));
            }

            if self.debug {
                println!(
                    "(LOADER) DEBUG: Plugin loaded! (name=\"{}\", version=\"{}\", module_count=\"{}\")",
                    plugin.plugin.get_name(),
                    plugin.plugin.get_version(),
                    plugin.plugin.get_modules().len()
                );
            }

            let mut module_id = PLUGIN_MODULE_BASE;
            for module in plugin.plugin.get_modules() {
                if self.debug {
                    println!(
                        "(LOADER) DEBUG: Reading module #{}",
                        module_id - PLUGIN_MODULE_BASE
                    );
                }

                if module_id == u16::MAX {
                    return Err(LoaderError::ReachedPluginModuleLimit);
                }

                if modules
                    .values()
                    .any(|mdl: &Module| mdl.get_name() == module.get_name())
                {
                    return Err(LoaderError::ModuleNameAlreadyUsed(module.get_name().into()));
                }

                if module.get_functions().len() > MAX_FUNCTIONS as usize {
                    return Err(LoaderError::ReachedFunctionLimit);
                }

                if self.debug {
                    println!(
                        "(LOADER) DEBUG: Module info read! (name=\"{}\", function_count=\"{}\")",
                        module.get_name(),
                        module.get_functions().len()
                    );
                }

                let mut functions = HashMap::with_capacity(module.get_functions().len());
                let mut function_id = 0;
                for function in module.get_functions() {
                    if self.debug {
                        println!("(LOADER) DEBUG: Reading function #{}", function_id);
                    }

                    if function.get_parameter_count() > MAX_FUNCTION_PARAMETERS {
                        return Err(LoaderError::ReachedFunctionParameterLimit);
                    }

                    if functions
                        .values()
                        .any(|func: &Function| func.get_name() == function.get_name())
                    {
                        return Err(LoaderError::FunctionNameAlreadyUsed {
                            module_name: module.get_name().into(),
                            name: function.get_name().into(),
                        });
                    }

                    if self.debug {
                        println!(
                            "(LOADER) DEBUG: Function info read! (name=\"{}\", parameter_count=\"{}\")",
                            function.get_name(),
                            function.get_parameter_count()
                        );
                    }

                    let handler = function.get_handler();

                    functions.insert(
                        function_id,
                        Function::new(
                            function.get_name().into(),
                            function.get_parameter_count(),
                            Arc::new(move |vm: &mut Vm, parameters: &[Value]| -> CandleResult<Option<Value>> {
                                let result = handler(vm, parameters);
                                match result {
                                    Ok(value) => Ok(Some(value)),
                                    Err(err) => Err(err)
                                }
                            }),
                            true,
                        ),
                    );

                    function_id += 1;
                }

                modules.insert(module_id, Module::new(module.get_name().into(), functions));

                module_id += 1;
            }

            plugins.push(plugin);

            plugin_id += 1;
        }
        Ok((modules, plugins))
    }

    fn check_plugins(&mut self, plugins: &Vec<LoadedPlugin>) -> LoaderResult<()> {
        if self.debug {
            println!("(LOADER) DEBUG: Reading plugin dependency table");
        }

        let plugin_count = self.fetch_u16()?;
        for i in 0..plugin_count {
            let plugin_name = self.fetch_string()?;
            let plugin_version = self.fetch_string()?;

            if self.debug {
                println!(
                    "(LOADER) Read dependency #{} (name=\"{}\", version=\"{}\")",
                    i, plugin_name, plugin_version
                );
            }

            if !plugins.iter().any(|plugin: &LoadedPlugin| {
                plugin.plugin.get_name().clone() == plugin_name
                    && plugin.plugin.get_version().clone() == plugin_version
            }) {
                return Err(LoaderError::MissingPlugin {
                    name: plugin_name,
                    version: plugin_version,
                });
            }
        }
        Ok(())
    }

    fn load_modules(&mut self, modules: &mut HashMap<u16, Module>) -> LoaderResult<()> {
        if self.debug {
            println!("(LOADER) DEBUG: Reading module table");
        }

        let module_count = self.fetch_u16()?;
        if module_count > MODULE_SPACE_SIZE {
            return Err(LoaderError::ReachedModuleLimit);
        }

        for module_id in 0..module_count {
            if self.debug {
                println!("(LOADER) DEBUG: Reading module #{}", module_id);
            }

            let module_name = self.fetch_string()?;
            let function_count = self.fetch_u16()? as usize;

            if modules
                .values()
                .any(|module| module.get_name().clone() == module_name)
            {
                return Err(LoaderError::ModuleNameAlreadyUsed(module_name));
            }

            if function_count > MAX_FUNCTIONS as usize {
                return Err(LoaderError::ReachedFunctionLimit);
            }

            if self.debug {
                println!(
                    "(LOADER) DEBUG: Module info read! (name=\"{}\", function_count={})",
                    module_name, function_count
                );
            }

            let mut functions = HashMap::with_capacity(function_count);
            for function_id in 0..function_count {
                if self.debug {
                    println!("(LOADER) DEBUG: Reading function #{}", function_id);
                }

                let function_name = self.fetch_string()?;
                let entry = self.fetch_u32()?;
                let parameter_count = self.fetch_u8()?;
                let capture_count = self.fetch_u8()?;

                if functions
                    .values()
                    .any(|function: &Function| function.get_name().clone() == function_name)
                {
                    return Err(LoaderError::FunctionNameAlreadyUsed {
                        module_name: module_name.clone(),
                        name: function_name,
                    });
                }

                if parameter_count > MAX_FUNCTION_PARAMETERS {
                    return Err(LoaderError::ReachedFunctionParameterLimit);
                }
                if capture_count > MAX_CAPTURE_COUNT {
                    return Err(LoaderError::ReachedFunctionCaptureLimit);
                }

                let handler = Arc::new(
                    move |vm: &mut Vm,
                          parameters: &[Value]|
                          -> CandleResult<Option<Value>> {
                        for (i, parameter) in parameters.iter().enumerate() {
                            vm.set_register(i + 1, *parameter);
                        }

                        vm.set_pc(entry as usize);
                        Ok(None)
                    },
                );

                functions.insert(
                    function_id as u16,
                    Function::new_with_captures(
                        function_name.clone(),
                        parameter_count,
                        capture_count,
                        handler,
                        false,
                    ),
                );

                if self.debug {
                    println!(
                        "(LOADER) DEBUG: Function read! (name=\"{}\", entry=0x{:08X}, parameter_count={}, capture_count={})",
                        function_name, entry, parameter_count, capture_count
                    );
                }
            }

            modules.insert(module_id, Module::new(module_name, functions));
        }
        Ok(())
    }

    fn load_resources(&mut self) -> LoaderResult<HashMap<String, Vec<u8>>> {
        if self.debug {
            println!("(LOADER) DEBUG: Reading resource table");
        }

        let resource_count = self.fetch_u16()? as usize;
        if resource_count > MAX_RESOURCES as usize {
            return Err(LoaderError::ReachedResourceLimit);
        }

        let mut resources = HashMap::with_capacity(resource_count);

        for resource_id in 0..resource_count {
            if self.debug {
                println!("(LOADER) DEBUG: Reading resource #{}", resource_id);
            }

            let id = self.fetch_string()?;

            if resources
                .keys()
                .any(|res_id: &String| res_id.clone() == id)
            {
                return Err(LoaderError::ResourceIdAlreadyUsed(id));
            }

            let size = self.fetch_u32()? as usize;
            if size > MAX_RESOURCE_SIZE as usize {
                return Err(LoaderError::ReachedResourceSizeLimit);
            }

            let data = self.fetch_vec(size)?;

            if self.debug {
                println!("(LOADER) DEBUG: Resource read! (id={}, size={} bytes)", id, size);
            }

            resources.insert(id, data);
        }

        Ok(resources)
    }

    fn fetch_bytes<const N: usize>(&mut self) -> LoaderResult<[u8; N]> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or(LoaderError::UnexpectedEndOfFile)?;

        let bytes = self
            .data
            .get(self.offset..end)
            .ok_or(LoaderError::UnexpectedEndOfFile)?
            .try_into()
            .map_err(|_| LoaderError::UnexpectedEndOfFile)?;

        self.offset = end;
        Ok(bytes)
    }

    fn fetch_u8(&mut self) -> LoaderResult<u8> {
        let value = *self
            .data
            .get(self.offset)
            .ok_or(LoaderError::UnexpectedEndOfFile)?;

        self.offset += 1;
        Ok(value)
    }

    fn fetch_vec(&mut self, len: usize) -> LoaderResult<Vec<u8>> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or(LoaderError::UnexpectedEndOfFile)?;

        let bytes = self
            .data
            .get(self.offset..end)
            .ok_or(LoaderError::UnexpectedEndOfFile)?
            .to_vec();

        self.offset = end;
        Ok(bytes)
    }

    fn fetch_u16(&mut self) -> LoaderResult<u16> {
        Ok(u16::from_le_bytes(self.fetch_bytes()?))
    }

    fn fetch_u32(&mut self) -> LoaderResult<u32> {
        Ok(u32::from_le_bytes(self.fetch_bytes()?))
    }

    fn fetch_string(&mut self) -> LoaderResult<String> {
        let len = self.fetch_u32()? as usize;
        Ok(String::from_utf8(self.fetch_vec(len)?).map_err(|_| LoaderError::InvalidString)?)
    }

    fn read_header(&mut self) -> LoaderResult<()> {
        if self.debug {
            println!("(LOADER) DEBUG: Reading header");
        }

        let magic = self.fetch_bytes::<4>()?;
        if magic != *BYTECODE_MAGIC {
            return Err(LoaderError::InvalidMagic {
                expected: *BYTECODE_MAGIC,
                found: magic,
            });
        }

        let version = self.fetch_u16()?;
        if version != CANDLE_VERSION {
            return Err(LoaderError::UnsupportedVersion {
                expected: CANDLE_VERSION,
                found: version,
            });
        }

        Ok(())
    }
}
