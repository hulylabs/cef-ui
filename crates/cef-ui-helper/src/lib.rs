mod app;
mod browser;
mod frame;
mod lib_loader;
mod macros;
mod main_args;
mod process;
mod refcounted;
mod render_process_handler;
mod run;
mod sandbox;
mod scheme;
mod string;
mod v8_context;
mod values;

pub use app::*;
pub use browser::*;
pub use frame::*;
pub use lib_loader::*;
pub use macros::*;
pub use main_args::*;
pub use process::*;
pub use refcounted::*;
pub use render_process_handler::*;
pub use run::*;
pub use sandbox::*;
pub use scheme::*;
pub use string::*;
pub use v8_context::*;
pub use values::*;

use cef_ui_sys::CEF_API_VERSION_14100;

pub fn cef_api_hash() {
    unsafe {
        let lib = &CEFLIB;
        (lib.cef_api_hash)(CEF_API_VERSION_14100 as i32, 0);
    }
}

pub fn execute_process(args: MainArgs, app: Option<App>) -> i32 {
    unsafe {
        let lib = &CEFLIB;
        (lib.cef_execute_process)(
            args.as_raw(),
            app.map(|a| a.as_ptr())
                .unwrap_or(std::ptr::null_mut()),
            std::ptr::null_mut()
        )
    }
}

pub fn register_extension(name: &str, code: &str, handler: Option<V8Handler>) {
    unsafe {
        let lib = &CEFLIB;
        (lib.cef_register_extension)(
            CefString::new(name).as_ptr(),
            CefString::new(code).as_ptr(),
            handler
                .map(|h| h.as_ptr())
                .unwrap_or(std::ptr::null_mut())
        );
    }
}
