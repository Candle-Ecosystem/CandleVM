use crate::callstack::CallFrame;
use crate::constants::{ALLOCATIONS_BEFORE_GC, CANDLE_VERSION, MAX_STACKTRACE_FRAMES, VM_VERSION};
use crate::error::{CandleError, CandleResult};
use crate::gc::GarbageCollector;
use crate::globals::Global;
use crate::heap::Object;
use crate::heap::{Heap, ObjectType};
use crate::module::Module;
use crate::opcode::Opcode;
use crate::value::{Value, ValueType};
use std::collections::HashMap;
use std::process::exit;

pub struct Vm {
    modules: HashMap<u16, Module>,
    resources: HashMap<String, Vec<u8>>,
    bytecode: Vec<u8>,

    pc: usize,

    registers: Vec<Value>,
    globals: HashMap<String, Global>,
    heap: Heap,

    call_stack: Vec<CallFrame>,

    gc: GarbageCollector,

    last_opcode: Option<Opcode>,

    running: bool,
    exit_code: u8,

    debug: bool,
}

impl Vm {
    pub fn new(
        bytecode: Vec<u8>,
        modules: HashMap<u16, Module>,
        resources: HashMap<String, Vec<u8>>,
        debug: bool,
    ) -> Self {
        Self {
            modules,
            bytecode,
            resources,
            pc: 0,
            registers: Vec::new(),
            globals: HashMap::new(),
            heap: Heap::new(debug),
            call_stack: Vec::new(),
            gc: GarbageCollector::new(debug),
            last_opcode: None,
            running: true,
            exit_code: 0,
            debug,
        }
    }

    pub fn run(&mut self) -> u8 {
        ctrlc::set_handler(|| {
            eprintln!("\nReceived Ctrl+C, terminating CandleVM...");
            exit(130);
        }).expect("Failed to set Ctrl+C handler");

        self.init_builtins();

        while self.running {
            if let Err(err) = self.step() {
                self.handle_error(err);
            }
        }

        self.exit_code
    }

    fn init_builtins(&mut self) {
        let object_id = self.get_heap().allocate(Object::String(VM_VERSION.into()));
        self.set_global("API_VERSION".into(), Value::Number(CANDLE_VERSION.into()), true).unwrap();
        self.set_global("VM_VERSION".into(), Value::ObjectRef(object_id), true).unwrap();

        self.set_global("DEBUG".into(), Value::Bool(self.debug), true).unwrap();

        self.set_global("TYPE_NULL".into(), Value::Byte(ValueType::Null as u8), true).unwrap();
        self.set_global("TYPE_BYTE".into(), Value::Byte(ValueType::Byte as u8), true).unwrap();
        self.set_global("TYPE_NUMBER".into(), Value::Byte(ValueType::Number as u8), true).unwrap();
        self.set_global("TYPE_BOOL".into(), Value::Byte(ValueType::Bool as u8), true).unwrap();
        self.set_global("TYPE_CHAR".into(), Value::Byte(ValueType::Char as u8), true).unwrap();
        self.set_global("TYPE_MODULE_REF".into(), Value::Byte(ValueType::ModuleRef as u8), true).unwrap();
        self.set_global("TYPE_FUNCTION_REF".into(), Value::Byte(ValueType::FunctionRef as u8), true).unwrap();
        self.set_global("TYPE_OBJECT_REF".into(), Value::Byte(ValueType::ObjectRef as u8), true).unwrap();

        self.set_global("OBJECT_TYPE_ARRAY".into(), Value::Byte(ObjectType::Array as u8), true).unwrap();
        self.set_global("OBJECT_TYPE_MAP".into(), Value::Byte(ObjectType::Map as u8), true).unwrap();
        self.set_global("OBJECT_TYPE_STRING".into(), Value::Byte(ObjectType::String as u8), true).unwrap();
        self.set_global("OBJECT_TYPE_ERROR".into(), Value::Byte(ObjectType::Error as u8), true).unwrap();
        self.set_global("OBJECT_TYPE_BUFFER".into(), Value::Byte(ObjectType::Buffer as u8), true).unwrap();
        self.set_global("OBJECT_TYPE_CLOSURE".into(), Value::Byte(ObjectType::Closure as u8), true).unwrap();
        self.set_global("OBJECT_TYPE_NATIVE".into(), Value::Byte(ObjectType::Native as u8), true).unwrap();
    }

    fn step(&mut self) -> CandleResult<()> {
        let opcode = self.fetch_opcode()?;
        self.last_opcode = Some(opcode);

        if self.debug {
            println!(
                "(VM) DEBUG: 0x{:08X} | {:?} (0x{:02X})",
                self.pc, opcode, opcode as u8
            );
        }

        match opcode {
            Opcode::Nop => self.exec_nop(),
            Opcode::Exit => self.exec_exit(),
            Opcode::Call => self.exec_call(),
            Opcode::Pcall => self.exec_pcall(),
            Opcode::Ret => self.exec_ret(),
            Opcode::Throw => self.exec_throw(),
            Opcode::Jump => self.exec_jump(),
            Opcode::JumpIf => self.exec_jump_if(),
            Opcode::JumpIfNot => self.exec_jump_if_not(),

            Opcode::LoadConst => self.exec_load_const(),
            Opcode::Mov => self.exec_mov(),
            Opcode::LoadModule => self.exec_load_module(),
            Opcode::LoadFunction => self.exec_load_function(),
            Opcode::LoadGlobal => self.exec_load_global(),
            Opcode::StoreGlobal => self.exec_store_global(),
            Opcode::LoadResource => self.exec_load_resource(),

            Opcode::Type => self.exec_type(),
            Opcode::ObjectType => self.exec_object_type(),

            Opcode::Add => self.exec_add(),
            Opcode::Sub => self.exec_sub(),
            Opcode::Mul => self.exec_mul(),
            Opcode::Div => self.exec_div(),
            Opcode::Mod => self.exec_mod(),
            Opcode::Inc => self.exec_inc(),
            Opcode::Dec => self.exec_dec(),

            Opcode::Eq => self.exec_eq(),
            Opcode::Ne => self.exec_ne(),
            Opcode::ObjectEq => self.exec_object_eq(),
            Opcode::ObjectNe => self.exec_object_ne(),
            Opcode::Lt => self.exec_lt(),
            Opcode::Lte => self.exec_lte(),
            Opcode::Gt => self.exec_gt(),
            Opcode::Gte => self.exec_gte(),

            Opcode::And => self.exec_and(),
            Opcode::Or => self.exec_or(),
            Opcode::Xor => self.exec_xor(),
            Opcode::Not => self.exec_not(),

            Opcode::NewArray => self.exec_new_array(),
            Opcode::ArrayLen => self.exec_array_len(),
            Opcode::ArrayContains => self.exec_array_contains(),
            Opcode::ArrayGet => self.exec_array_get(),
            Opcode::ArraySet => self.exec_array_set(),
            Opcode::ArrayInsert => self.exec_array_insert(),
            Opcode::ArrayRemove => self.exec_array_remove(),

            Opcode::NewMap => self.exec_new_map(),
            Opcode::MapLen => self.exec_map_len(),
            Opcode::MapContains => self.exec_map_contains(),
            Opcode::MapGet => self.exec_map_get(),
            Opcode::MapSet => self.exec_map_set(),
            Opcode::MapRemove => self.exec_map_remove(),

            Opcode::NewString => self.exec_new_string(),
            Opcode::StringLen => self.exec_string_len(),
            Opcode::StringConcat => self.exec_string_concat(),
            Opcode::StringFromChars => self.exec_string_from_chars(),
            Opcode::StringToChars => self.exec_string_to_chars(),
            Opcode::StringFromBytes => self.exec_string_from_bytes(),
            Opcode::StringToBytes => self.exec_string_to_bytes(),
            Opcode::StringOf => self.exec_string_of(),
            Opcode::StringLowercase => self.exec_string_lowercase(),
            Opcode::StringUppercase => self.exec_string_uppercase(),
            Opcode::StringToNumber => self.exec_string_to_number(),

            Opcode::NewError => self.exec_new_error(),
            Opcode::ErrorType => self.exec_error_type(),
            Opcode::ErrorMessage => self.exec_error_message(),

            Opcode::NewBuffer => self.exec_new_buffer(),
            Opcode::BufferLen => self.exec_buffer_len(),
            Opcode::BufferGet => self.exec_buffer_get(),
            Opcode::BufferSet => self.exec_buffer_set(),
            Opcode::NewClosure => self.exec_new_closure(),
        }
    }

    pub fn stop(&mut self, exit_code: u8) {
        self.running = false;
        self.exit_code = exit_code;
    }

    pub fn handle_error(&mut self, err: CandleError) {
        if let Err((frames, registers)) = self.unwind(&err) {
            self.stop(1);
            self.report_unhandled_error(&err, &frames, &registers);
        }
    }

    fn unwind(&mut self, err: &CandleError) -> Result<(), (Vec<CallFrame>, Vec<Value>)> {
        let frames = self.call_stack.clone();
        let mut registers = self.registers.clone();

        let register_base = frames.last().map_or(0, |frame| frame.get_register_base());

        registers.drain(..register_base);

        while let Some(frame) = self.call_stack.pop() {
            self.registers.truncate(frame.get_register_base());

            self.set_register(frame.get_return_register(), Value::Null);

            self.set_pc(frame.get_return_pc());

            if let Some(error_register) = frame.get_error_register() {
                let id = self.heap.allocate(Object::Error(err.clone()));

                self.set_register(error_register, Value::ObjectRef(id));

                return Ok(());
            }
        }

        Err((frames, registers))
    }

    fn report_unhandled_error(
        &mut self,
        err: &CandleError,
        frames: &[CallFrame],
        registers: &[Value],
    ) {
        let stacktrace_truncated = frames.len() > MAX_STACKTRACE_FRAMES;

        let callstack = frames
            .iter()
            .rev()
            .take(MAX_STACKTRACE_FRAMES)
            .map(|frame| {
                let module_id = frame.get_function_module();
                let function_id = frame.get_function_id();

                match self.get_module(module_id) {
                    Some(module) => {
                        let function_name = module
                            .get_function(function_id)
                            .map(|function| function.get_name().into())
                            .unwrap_or_else(|| format!("<function_{}>", function_id));

                        if frame.is_native() {
                            format!("   at [NATIVE] {}::{}", module.get_name(), function_name)
                        } else {
                            format!("   at {}::{}", module.get_name(), function_name)
                        }
                    }

                    None => {
                        if frame.is_native() {
                            format!(
                                "   at [NATIVE] <unknown_module_{}>::<function_{}>",
                                module_id, function_id
                            )
                        } else {
                            format!(
                                "   at <unknown_module_{}>::<function_{}>",
                                module_id, function_id
                            )
                        }
                    }
                }
            })
            .chain(stacktrace_truncated.then(|| "   ... (stacktrace truncated)".into()))
            .chain(std::iter::once("   at <raw bytecode>".into()))
            .collect::<Vec<_>>()
            .join("\n");

        let instruction = if self.debug {
            Some(match self.last_opcode {
                Some(opcode) => format!(
                    "Instruction:\n   Last opcode: {:?} (0x{:02X})\n   PC: 0x{:08X}",
                    opcode, opcode as u8, self.pc
                ),
                None => format!(
                    "Instruction:\n   Last opcode: NONE\n   PC: 0x{:08X}",
                    self.pc
                ),
            })
        } else {
            None
        };

        let registers = if self.debug {
            if registers.is_empty() {
                Some("Registers:\n   NONE".into())
            } else {
                let index_width = registers.len().saturating_sub(1).to_string().len().max(1);

                Some(format!(
                    "Registers:\n{}",
                    registers
                        .iter()
                        .enumerate()
                        .map(|(i, value)| format!(
                            "   R{:<width$} │ {}",
                            i,
                            value,
                            width = index_width
                        ))
                        .collect::<Vec<_>>()
                        .join("\n")
                ))
            }
        } else {
            None
        };

        let title = " Unhandled Virtual Machine Error ";

        let stacktrace_title = format!(
            "Stacktrace ({} frame{}):",
            frames.len(),
            if frames.len() == 1 { "" } else { "s" }
        );

        let version = format!("VM version: {}", env!("CARGO_PKG_VERSION"));

        let width = std::iter::once(title)
            .chain(std::iter::once(err.to_string().as_str()))
            .chain(instruction.as_deref().into_iter().flat_map(str::lines))
            .chain(registers.as_deref().into_iter().flat_map(str::lines))
            .chain(stacktrace_title.lines())
            .chain(callstack.lines())
            .chain(version.lines())
            .map(str::len)
            .max()
            .unwrap_or(0)
            + 4;

        let separator = "=".repeat(width);

        eprintln!(
            "\n{}\n{:^width$}\n{}",
            separator,
            title,
            separator,
            width = width
        );

        eprintln!("{}", err);
        if let Some(instruction) = instruction {
            eprintln!("\n{}", instruction);
        }

        if let Some(registers) = registers {
            eprintln!("\n{}", registers);
        }

        eprintln!("\n{}", stacktrace_title);
        eprintln!("{}", callstack);
        eprintln!("\n{}", version);

        eprintln!("{}\n", separator);
    }

    pub fn fetch_bytes<const N: usize>(&mut self) -> CandleResult<[u8; N]> {
        let end = self
            .pc
            .checked_add(N)
            .ok_or(CandleError::UnexpectedBytecodeEnd)?;

        let bytes = self
            .bytecode
            .get(self.pc..end)
            .ok_or(CandleError::UnexpectedBytecodeEnd)?
            .try_into()
            .map_err(|_| CandleError::UnexpectedBytecodeEnd)?;

        self.pc = end;
        Ok(bytes)
    }

    pub fn fetch_u8(&mut self) -> CandleResult<u8> {
        let value = *self
            .bytecode
            .get(self.pc)
            .ok_or(CandleError::UnexpectedBytecodeEnd)?;

        self.pc += 1;
        Ok(value)
    }

    pub fn fetch_vec(&mut self, len: usize) -> CandleResult<Vec<u8>> {
        let end = self
            .pc
            .checked_add(len)
            .ok_or(CandleError::UnexpectedBytecodeEnd)?;

        let bytes = self
            .bytecode
            .get(self.pc..end)
            .ok_or(CandleError::UnexpectedBytecodeEnd)?
            .to_vec();

        self.pc = end;
        Ok(bytes)
    }

    fn fetch_opcode(&mut self) -> CandleResult<Opcode> {
        let byte = self.fetch_u8()?;
        Ok(Opcode::try_from(byte).map_err(|_| CandleError::InvalidOpcode(byte))?)
    }

    pub fn fetch_u16(&mut self) -> CandleResult<u16> {
        Ok(u16::from_le_bytes(self.fetch_bytes()?))
    }

    pub fn fetch_u32(&mut self) -> CandleResult<u32> {
        Ok(u32::from_le_bytes(self.fetch_bytes()?))
    }

    pub fn fetch_i32(&mut self) -> CandleResult<i32> {
        Ok(i32::from_le_bytes(self.fetch_bytes()?))
    }

    pub fn fetch_f64(&mut self) -> CandleResult<f64> {
        Ok(f64::from_le_bytes(self.fetch_bytes()?))
    }

    pub fn fetch_string(&mut self) -> CandleResult<String> {
        let len = self.fetch_u32()? as usize;
        Ok(String::from_utf8(self.fetch_vec(len)?).map_err(|_| CandleError::InvalidString)?)
    }

    pub fn fetch_value(&mut self) -> CandleResult<Value> {
        let value_type = self.fetch_u8()?;

        let value_type = ValueType::try_from(value_type)
            .map_err(|_| CandleError::InvalidValueType(value_type))?;

        match value_type {
            ValueType::Null => Ok(Value::Null),
            ValueType::Byte => Ok(Value::Byte(self.fetch_u8()?)),
            ValueType::Number => {
                let v = self.fetch_f64()?;
                Ok(Value::from_f64(v)?)
            }
            ValueType::Bool => Ok(Value::Bool(self.fetch_u8()? != 0)),
            ValueType::Char => Ok(Value::Char(
                char::from_u32(self.fetch_u32()?).ok_or(CandleError::InvalidChar)?,
            )),
            ValueType::ModuleRef | ValueType::FunctionRef | ValueType::ObjectRef => {
                Err(CandleError::ValueNotSerializable(value_type))
            }
        }
    }

    pub fn run_gc(&mut self) {
        if self.debug {
            println!(
                "(VM) DEBUG: Running GC ({} allocations since last GC)",
                self.heap.get_allocations_since_gc()
            );
        }

        let mut roots = Vec::new();
        roots.extend(self.registers.iter());
        roots.extend(self.globals.values().map(|global| &global.value));
        self.gc.collect(&mut self.heap, &roots);
    }

    pub fn maybe_run_gc(&mut self) {
        if self.heap.get_allocations_since_gc() >= ALLOCATIONS_BEFORE_GC {
            self.run_gc()
        }
    }

    pub fn get_bytecode(&self) -> &Vec<u8> {
        &self.bytecode
    }

    pub fn get_modules(&self) -> &HashMap<u16, Module> {
        &self.modules
    }

    pub fn get_module(&mut self, id: u16) -> Option<&mut Module> {
        for (module_id, module) in &mut self.modules {
            if *module_id == id {
                return Some(module);
            }
        }
        None
    }

    pub fn get_module_id(&mut self, name: String) -> Option<u16> {
        for (module_id, module) in &mut self.modules {
            if module.get_name().clone() == name {
                return Some(*module_id);
            }
        }
        None
    }

    pub fn get_pc(&self) -> usize {
        self.pc
    }

    pub fn set_pc(&mut self, new_pc: usize) {
        self.pc = new_pc
    }

    pub fn get_registers(&mut self) -> &mut Vec<Value> {
        &mut self.registers
    }

    pub fn get_register(&self, index: usize) -> Value {
        if index == 0 && let Some(call_frame) = self.call_stack.last() {
            return Value::ModuleRef(call_frame.get_function_module());
        }

        if index == 255 {
            return Value::Null;
        }

        let base = if let Some(frame) = self.call_stack.last() {
            frame.get_register_base()
        } else {
            0
        };

        let register = base + index;

        self.registers.get(register).cloned().unwrap_or(Value::Null)
    }

    pub fn set_register(&mut self, index: usize, value: Value) {
        if index == 0 && !self.call_stack.is_empty() {
            return;
        }

        if index == 255 {
            return;
        }

        let base = if let Some(frame) = self.call_stack.last() {
            frame.get_register_base()
        } else {
            0
        };

        let register = base + index;

        if register >= self.registers.len() {
            self.registers.resize(register + 1, Value::Null);
        }

        self.registers[register] = value;

        while self.registers.len() > base && matches!(self.registers.last(), Some(Value::Null)) {
            self.registers.pop();
        }
    }

    pub fn get_globals(&self) -> &HashMap<String, Global> {
        &self.globals
    }

    pub fn get_global(&self, name: String) -> Value {
        match self.globals.get(&name) {
            Some(global) => global.value.clone(),
            None => Value::Null,
        }
    }

    pub fn set_global(&mut self, name: String, value: Value, read_only: bool) -> CandleResult<()> {
        match self.globals.get(&name) {
            Some(global) => {
                match global.read_only {
                    true => Err(CandleError::ReadOnlyGlobal(name.clone())),
                    false => Ok(())
                }
            }
            None => Ok(())
        }?;

        if value == Value::Null {
            self.globals.remove(&name);
        } else {
            self.globals.insert(name, Global {
                value,
                read_only,
            });
        }

        Ok(())
    }

    pub fn get_resources(&self) -> &HashMap<String, Vec<u8>> {
        &self.resources
    }

    pub fn get_heap(&mut self) -> &mut Heap {
        &mut self.heap
    }

    pub fn get_call_stack(&mut self) -> &mut Vec<CallFrame> {
        &mut self.call_stack
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn is_debug(&self) -> bool {
        self.debug
    }
}
