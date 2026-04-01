use compiler_core::backend::Environ;

/// ...
#[repr(C)]
pub struct ExternEnviron {
    /// ...
    action_one: extern "C" fn(*const Self, u32),
    /// ...
    action_two: extern "C" fn(*const Self, u32),
}

impl Environ for ExternEnviron {
    /// ...
    fn action_one(&mut self, value: u32) {
        (self.action_one)(self as *const _, value);
    }

    /// ...
    fn action_two(&mut self, value: u32) {
        (self.action_two)(self as *const _, value);
    }
}
