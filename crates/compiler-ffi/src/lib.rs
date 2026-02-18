use std::ffi::{CString, c_char};

/// Returns a simple message.
#[unsafe(no_mangle)]
pub extern "C" fn greet() -> *const c_char {
    CString::new(compiler_core::greet()).unwrap().into_raw()
}
