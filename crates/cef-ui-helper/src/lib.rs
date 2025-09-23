mod app;
mod lib_loader;
mod main_args;
mod refcounted;
mod run;
mod sandbox;
mod scheme;
mod string;

pub use app::*;
pub use lib_loader::*;
pub use main_args::*;
pub use refcounted::*;
pub use run::*;
pub use sandbox::*;
pub use scheme::*;
pub use string::*;

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
