#![feature(rustc_private)]


mod llvm {
    use std::ffi::CString as CString;
    extern crate rustc_llvm;

    trait Type {
        fn pointer_to(&self) -> Type;
    }

    pub struct Module {
        _context: rustc_llvm::ContextRef,
        _module: rustc_llvm::ModuleRef
    }

    impl Module {
        pub fn new(name: &str) -> Module {
            unsafe {
                let context: rustc_llvm::ContextRef = rustc_llvm::LLVMContextCreate();
                let module: rustc_llvm::ModuleRef = rustc_llvm::LLVMModuleCreateWithNameInContext(CString::new(name).unwrap().as_ptr(), context);
                Module{ _context: context, _module: module }
            }
        }

        pub fn print(&self) {
            unsafe { rustc_llvm::LLVMDumpModule(self._module); }
        }
    }

    pub struct Builder<'a> {
        _module: &'a mut Module
    }

    impl<'a> Builder<'a> {
        pub fn new(module: &'a mut Module) -> Builder {
            Builder{ _module: module }
        }
    }

    pub struct FunctionType {
        returnType: Type,
        params: Vec<Type>
    }
}

fn main() {
/*    unsafe {
        let context: rustc_llvm::ContextRef = rustc_llvm::LLVMContextCreate();
        let module: rustc_llvm::ModuleRef = rustc_llvm::LLVMModuleCreateWithNameInContext(CString::new("test").unwrap().as_ptr(), context);
        rustc_llvm::LLVMDumpModule(module);
    }*/

//    println!("Hello, world!");
    let mut module = llvm::Module::new("ModuloDePrueba");
    {
        let builder = llvm::Builder::new(&mut module);
    }
    module.print();
}
