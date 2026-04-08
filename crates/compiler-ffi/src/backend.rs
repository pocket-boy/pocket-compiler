use compiler_core::{backend::Backend, input::InputState};

use crate::{CompilerHandle, environ::ExternEnviron};

/// ...
#[repr(transparent)]
pub struct BackendHandle(pub *mut Backend<ExternEnviron>);

/// ...
impl BackendHandle {
    /// ...
    #[unsafe(no_mangle)]
    unsafe extern "C" fn init_backend(compiler: CompilerHandle, environ: ExternEnviron) -> Self {
        // ...
        Self(Box::into_raw(Box::new(Backend::new(
            unsafe { compiler.0.as_ref().unwrap().clone() },
            environ,
        ))))
    }

    /// ...
    #[unsafe(no_mangle)]
    unsafe extern "C" fn drop_backend(self) {
        // ...
        unsafe { drop(Box::from_raw(self.0)) }
    }
}

impl BackendHandle {
    /// ...
    #[unsafe(no_mangle)]
    unsafe extern "C" fn backend_render(self, input: u8) {
        // ...
        let backend = unsafe { self.0.as_mut().unwrap() };
        // ...
        backend.render(InputState(input));
    }
}
