use crate::{CEFLIB, MainArgs, sandbox::ScopedSandbox};
use anyhow::Result;
use cef_ui_sys::CEF_API_VERSION_13800;
use std::{process::exit, ptr::null_mut};
use tracing::{Level, error, info, level_filters::LevelFilter, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

/// Returns the CEF error code or 1 if an error occurred.
pub fn run(sandbox: bool) {
    let ret = try_run(sandbox).unwrap_or_else(|e| {
        error!("An error occurred: {}", e);

        1
    });

    info!("The return code is: {}", ret);

    exit(ret);
}

/// Try and run the helper, returning the CEF error code if successful.
fn try_run(sandbox: bool) -> Result<i32> {
    // This routes log macros through tracing.
    LogTracer::init()?;

    // Setup the tracing subscriber globally.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::from_level(Level::DEBUG))
        .finish();

    set_global_default(subscriber)?;

    // Setup the sandbox if enabled.
    let _sandbox = match sandbox {
        true => Some(ScopedSandbox::new()?),
        false => None
    };

    // Manually load CEF and execute the subprocess.
    let ret = unsafe {
        let main_args = MainArgs::new()?;
        let lib = &CEFLIB;
        (lib.cef_api_hash)(CEF_API_VERSION_13800 as i32, 0);
        (lib.cef_execute_process)(main_args.as_raw(), null_mut(), null_mut()) as i32
    };

    Ok(ret)
}
