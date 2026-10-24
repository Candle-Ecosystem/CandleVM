#[derive(Debug, Clone)]
pub struct CallFrame {
    function_module: u16,
    function_id: u16,
    return_pc: usize,
    return_register: usize,
    error_register: Option<usize>,
    register_base: usize,
    native: bool,
}

impl CallFrame {
    pub fn new(
        function_module: u16,
        function_id: u16,
        return_pc: usize,
        return_register: usize,
        error_register: Option<usize>,
        register_base: usize,
        native: bool,
    ) -> Self {
        Self {
            function_module,
            function_id,
            return_pc,
            return_register,
            error_register,
            register_base,
            native,
        }
    }

    pub fn get_function_module(&self) -> u16 {
        self.function_module
    }

    pub fn get_function_id(&self) -> u16 {
        self.function_id
    }

    pub fn get_return_pc(&self) -> usize {
        self.return_pc
    }

    pub fn get_return_register(&self) -> usize {
        self.return_register
    }

    pub fn get_error_register(&self) -> Option<usize> {
        self.error_register
    }

    pub fn get_register_base(&self) -> usize {
        self.register_base
    }

    pub fn is_native(&self) -> bool {
        self.native
    }
}
