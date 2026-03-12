#![allow(clippy::missing_safety_doc)]

use std::{
    collections::HashMap,
    ffi::{CStr, c_char},
};

pub struct CompilerHandle(pub *mut compiler_core::Compiler);

pub struct ResultHandle(pub *mut HashMap<String, Result<String, String>>);

impl CompilerHandle {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn drop_module(&mut self, name: *const c_char) -> bool {
        unsafe {
            self.0
                .as_mut()
                .and_then(|compiler| {
                    let name = CStr::from_ptr(name).to_str().ok()?;
                    compiler.drop_module(name)
                })
                .is_some()
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn load_module(
        &mut self,
        name: *const c_char,
        content: *const c_char,
    ) -> bool {
        unsafe {
            self.0
                .as_mut()
                .and_then(|compiler| {
                    let name = CStr::from_ptr(name).to_str().ok()?;
                    let content = CStr::from_ptr(content).to_str().ok()?;
                    compiler.load_module(name, content)
                })
                .is_some()
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn bind_module(
        &mut self,
        name: *const c_char,
        content: *const c_char,
    ) -> bool {
        unsafe {
            self.0
                .as_mut()
                .and_then(|compiler| {
                    let name = CStr::from_ptr(name).to_str().ok()?;
                    let content = CStr::from_ptr(content).to_str().ok()?;
                    compiler.bind_module(name, content)
                })
                .is_some()
        }
    }
}

impl ResultHandle {}
