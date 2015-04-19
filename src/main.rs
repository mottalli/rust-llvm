#![feature(rustc_private)]
#![feature(libc)]
use std::mem;

mod llvm {
    extern crate rustc_llvm;
    extern crate libc;

    use std::ffi::CString;
    use std::ffi::CStr;
    use std::mem;
    use std::str;

    /*************************************************************************/
    extern fn __morestack() {
        println!("REQUESTED MORE STACK! SHOULD NOT HAPPEN!");
    }

    /*************************************************************************/
    pub trait LLVMType {
        fn to_llvm_type(&self) -> rustc_llvm::TypeRef;
    }
    
    /*************************************************************************/
    pub trait LLVMValue {
        fn to_llvm_value(&self) -> rustc_llvm::ValueRef;
        fn get_type(&self) -> Type; 

        fn get_llvm_type(&self) -> rustc_llvm::TypeRef {
            self.get_type().to_llvm_type()
        }
    }

    /*************************************************************************/
    pub struct Module {
        _context: rustc_llvm::ContextRef,
        _module: rustc_llvm::ModuleRef
    }

    /*************************************************************************/
    pub struct Builder {
        _builder: rustc_llvm::BuilderRef
    }

    /*************************************************************************/
    pub struct Type {
        _type: rustc_llvm::TypeRef
    }

    impl LLVMType for Type {
        fn to_llvm_type(&self) -> rustc_llvm::TypeRef { self._type }
    }

    /*************************************************************************/
    pub struct FunctionType {
        _type: rustc_llvm::TypeRef
    }

    impl LLVMType for FunctionType {
        fn to_llvm_type(&self) -> rustc_llvm::TypeRef { self._type }
    }

    /*************************************************************************/
    pub struct Function {
        _value: rustc_llvm::ValueRef,
        _module: *mut Module
    }

    /*************************************************************************/
    pub struct BasicBlock {
        _block: rustc_llvm::BasicBlockRef,
        _function: *mut Function
    }

    /*************************************************************************/
    pub struct Value {
        _value: rustc_llvm::ValueRef
    }

    /*************************************************************************/
    pub struct ExecutionEngine {
        _engine: rustc_llvm::ExecutionEngineRef,
        _module: Module
    }

    /*************************************************************************/
    fn _get_string_ptr(s: &str) -> *const libc::c_char { CString::new(s).unwrap().as_ptr() }

    /*************************************************************************/
    impl Module {
        pub fn new(name: &str) -> Module {
            unsafe {
                let context: rustc_llvm::ContextRef = rustc_llvm::LLVMContextCreate();
                let module: rustc_llvm::ModuleRef = rustc_llvm::LLVMModuleCreateWithNameInContext(_get_string_ptr(name), context);
                Module{ _context: context, _module: module }
            }
        }

        pub fn get_int32_type(&mut self) -> Type { unsafe { Type{ _type: rustc_llvm::LLVMInt32TypeInContext(self._context) } } }

        pub fn create_function_type(&mut self, return_type: &LLVMType, param_types: &[&LLVMType]) -> FunctionType {
            let llvm_types: Vec<rustc_llvm::TypeRef> = param_types.iter().map(|t| t.to_llvm_type()).collect();
            let p_types: *const rustc_llvm::TypeRef = llvm_types.as_ptr();
            unsafe {
                FunctionType{ _type: rustc_llvm::LLVMFunctionType(return_type.to_llvm_type(), p_types, param_types.len() as libc::c_uint, rustc_llvm::False) }
            }
        }

        pub fn add_function(&mut self, name: &str, func_type: &FunctionType) -> Function {
            unsafe { Function { 
                _value: rustc_llvm::LLVMAddFunction(self._module, _get_string_ptr(name), func_type.to_llvm_type()),
                _module: self
            } }
        }

        pub fn print(&self) { unsafe { rustc_llvm::LLVMDumpModule(self._module); } }
    }

    /*************************************************************************/
    impl Function {
        pub fn create_basic_block(&mut self, name: &str) -> BasicBlock {
            unsafe { BasicBlock{ 
                _block: rustc_llvm::LLVMAppendBasicBlockInContext((*self._module)._context, self._value, _get_string_ptr(name)),
                _function: self
            } }
        }
    }

    /*************************************************************************/
    impl BasicBlock {
        pub fn build<F>(&mut self, builder_fn: F)
            where F: Fn(&mut Builder) {
            unsafe {
                let mut builder = Builder::new((*self._function)._module);
                builder.set_insert_point(self);
                builder_fn(&mut builder);
            }
        }
    }

    /*************************************************************************/
    impl Builder {
        pub fn new(module: *mut Module) -> Builder {
            unsafe {
                let context = (*module)._context;
                Builder{ _builder: rustc_llvm::LLVMCreateBuilderInContext(context) }
            }
        }

        /*pub fn create_function(&mut self, name: &str, func_type: &FunctionType) -> Function {
            unsafe { Function{ _value: rustc_llvm::LLVMAddFunction(self._module._module, _get_string_ptr(name), func_type._type) } }
        }*/

        /*pub fn create_basic_block(&mut self, name: &str, func: &mut Function) -> BasicBlock {
            unsafe { BasicBlock{ _block: rustc_llvm::LLVMAppendBasicBlockInContext(self._module._context, func._value, _get_string_ptr(name)) } }
        }*/

        pub fn set_insert_point(&mut self, block: &BasicBlock) {
            unsafe { rustc_llvm::LLVMPositionBuilderAtEnd(self._builder, block._block); }
        }

        pub fn create_add(&mut self, lhs: &Value, rhs: &Value) -> Value {
            unsafe { Value{ _value: rustc_llvm::LLVMBuildAdd(self._builder, lhs._value, rhs._value, _get_string_ptr("sum")) } }
        }

        pub fn create_ret(&mut self, val: &Value) {
            unsafe { rustc_llvm::LLVMBuildRet(self._builder, val._value); }
        }

        pub fn get_param(&mut self, function: &Function, index: u32) -> Value {
            Value{ _value: rustc_llvm::get_param(function._value, index as libc::c_uint) }
        }
    }

    /*************************************************************************/
    impl ExecutionEngine {
        pub fn new(module: Module) -> ExecutionEngine {
            unsafe {
                let morestack: *const () = mem::transmute(__morestack);
                let jit_memory_manager = rustc_llvm::LLVMRustCreateJITMemoryManager(morestack);
                // Is this OK?
                let engine = ExecutionEngine{ _engine: rustc_llvm::LLVMBuildExecutionEngine(module._module, jit_memory_manager), _module: module };
                let c_error = rustc_llvm::LLVMRustGetLastError();
                if !c_error.is_null() {
                    let error = CStr::from_ptr(c_error);
                    println!("Error dice: {}", str::from_utf8(error.to_bytes()).unwrap());
                }
                engine
            }
        }

        pub fn finalize(&mut self) -> &mut ExecutionEngine {
            unsafe { rustc_llvm::LLVMExecutionEngineFinalizeObject(self._engine); self }
        }

        pub fn get_pointer_to_function(&self, function: &Function) -> *const() {
            unsafe { rustc_llvm::LLVMGetPointerToGlobal(self._engine, function._value) }
        }
    }
}

/*************************************************************************/
fn main() {
    let mut module = llvm::Module::new("MainModule");

    let return_type = module.get_int32_type();
    let param1_type = module.get_int32_type();
    let param2_type = module.get_int32_type();
    let function_type = module.create_function_type(&return_type, &[&param1_type, &param2_type]);
    let mut function = module.add_function("foo", &function_type);
    
    function.create_basic_block("init").build(|builder| {
        let param1 = builder.get_param(&function, 0);
        let param2 = builder.get_param(&function, 1);
        let sum = builder.create_add(&param1, &param2);
        builder.create_ret(&sum);
    });

    module.print();

    let mut engine = llvm::ExecutionEngine::new(module);
    engine.finalize();
    let fptr: *const() = engine.get_pointer_to_function(&function);
    assert!(!fptr.is_null());
    unsafe {
        println!("Function pointer: {:?}", fptr);
        println!("Running JITted function...");
        let foo: fn(i32, i32) -> i32 = mem::transmute(fptr);
        let res = foo(12, 13);
        println!("res: {}", res);
    }
}
