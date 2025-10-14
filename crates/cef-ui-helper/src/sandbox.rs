use crate::lib_loader::CEF_SANDBOX_LIB;
use anyhow::{Result, anyhow};
use std::{
    env::args,
    ffi::{CString, c_char, c_int},
    os::raw::c_void
};
use tracing::info;

/// Declaring this will initialize the sandbox and
/// keep it active until the object is dropped.
pub struct ScopedSandbox {
    /// The sandbox context.
    context: *mut c_void
}

impl ScopedSandbox {
    pub fn new() -> Result<Self> {
        let args = args()
            .into_iter()
            .map(|arg| CString::new(arg))
            .collect::<Result<Vec<CString>, _>>()?;
        let argv = args
            .iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const c_char>>();

        let context = unsafe {
            (CEF_SANDBOX_LIB.cef_sandbox_initialize)(
                argv.len() as c_int,
                argv.as_ptr() as *mut *mut c_char
            )
        };

        match context.is_null() {
            true => Err(anyhow!("Failed to initialize sandbox!")),
            false => {
                info!("Sandbox initialized!");

                Ok(Self { context })
            }
        }
    }
}

impl Drop for ScopedSandbox {
    fn drop(&mut self) {
        unsafe { (CEF_SANDBOX_LIB.cef_sandbox_destroy)(self.context) };

        info!("Sandbox destroyed!");
    }
}
