#![allow(clippy::missing_safety_doc)]

pub mod backend;
pub mod environ;

use std::{
    collections::HashMap,
    ffi::{CStr, CString, c_char},
    ptr,
};

#[repr(transparent)]
pub struct CompilerHandle(pub *mut compiler_core::compiler::Compiler);

#[repr(transparent)]
pub struct ResultHandle(pub *mut HashMap<String, Result<String, String>>);

#[repr(transparent)]
pub struct StringHandle(pub *mut c_char);

impl CompilerHandle {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn init_compiler() -> CompilerHandle {
        Self(Box::into_raw(Box::default()))
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn drop_compiler(self) {
        unsafe { drop(Box::from_raw(self.0)) }
    }
}

impl ResultHandle {
    pub unsafe fn new(inner: HashMap<String, Result<String, String>>) -> Self {
        Self(Box::into_raw(Box::new(inner)))
    }
}

impl ResultHandle {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn drop_result(self) {
        unsafe { drop(Box::from_raw(self.0)) }
    }
}

impl StringHandle {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn drop_string(self) {
        unsafe { drop(CString::from_raw(self.0)) }
    }
}

impl CompilerHandle {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn drop_module(self, name: *const c_char) -> bool {
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
        self,
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
        self,
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

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn try_build(self) -> ResultHandle {
        unsafe {
            self.0
                .as_mut()
                .map(|compiler| ResultHandle::new(compiler.try_build()))
                .unwrap_or_else(|| ResultHandle(ptr::null_mut()))
        }
    }
}

impl ResultHandle {
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn result_module(
        self,
        name: *const c_char,
        err: *mut bool,
    ) -> StringHandle {
        unsafe {
            let name = CStr::from_ptr(name).to_str().ok();
            StringHandle(
                name.and_then(|name| self.0.as_ref().and_then(|results| results.get(name)))
                    .and_then(|result| {
                        CString::new(
                            match result {
                                Ok(item) => {
                                    *err = false;
                                    item
                                }
                                Err(error) => {
                                    *err = true;
                                    error
                                }
                            }
                            .clone(),
                        )
                        .ok()
                    })
                    .unwrap_or_else(|| {
                        CString::new("module missing or invalid compiler handle").unwrap()
                    })
                    .into_raw(),
            )
        }
    }
}
