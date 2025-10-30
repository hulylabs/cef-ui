use crate::{
    App, AppCallbacks, Browser, CEFLIB, Frame, MainArgs, RenderProcessHandlerCallbacks, V8Context,
    V8Handler, V8HandlerCallbacks, V8Value, cef_api_hash, render_process_handler,
    sandbox::ScopedSandbox
};
use anyhow::Result;
use std::{process::exit, ptr::null_mut};
use tracing::{Level, error, info, level_filters::LevelFilter, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

/// Returns the CEF error code or 1 if an error occurred.
pub fn run(sandbox: bool) {
    let app = App::new(MyAppCallbacks {});
    let ret = try_run(sandbox, app.clone()).unwrap_or_else(|e| {
        error!("An error occurred: {}", e);

        1
    });

    info!("The return code is: {}", ret);

    exit(ret);
}

/// Try and run the helper, returning the CEF error code if successful.
fn try_run(sandbox: bool, app: App) -> Result<i32> {
    // This routes log macros through tracing.
    LogTracer::init()?;

    cef_api_hash();

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

        // Manually load the CEF framework.
        let lib = &CEFLIB;

        // Execute the CEF subprocess.
        let ret = (lib.cef_execute_process)(main_args.as_raw(), app.as_ptr(), null_mut()) as i32;

        ret
    };

    Ok(ret)
}

struct MyAppCallbacks {}

impl AppCallbacks for MyAppCallbacks {
    fn get_render_process_handler(
        &mut self
    ) -> Option<render_process_handler::RenderProcessHandler> {
        Some(render_process_handler::RenderProcessHandler::new(
            RenderProcessCallbacks {
                func:    None,
                handler: None
            }
        ))
    }
}

pub struct RenderProcessCallbacks {
    func:    Option<V8Value>,
    handler: Option<V8Handler>
}

impl RenderProcessHandlerCallbacks for RenderProcessCallbacks {
    fn on_context_created(&mut self, _browser: Browser, frame: Frame, context: V8Context) {
        // let object = context.get_global().unwrap();

        let handler = V8Handler::new(SendMessageHandler::new());

        // // Use unsafe to create function and manually manage the handler pointer
        // let func = unsafe {
        //     let lib = &crate::CEFLIB;
        //     let name_str = crate::CefString::new("myFunc");
        //     // Transfer ownership of handler to CEF using into_raw()
        //     let func_ptr =
        //         (lib.cef_v8_value_create_function)(name_str.as_ptr(), handler.clone().into_raw());
        //     V8Value::from_ptr_unchecked(func_ptr)
        // };

        // // Store references to prevent premature dropping
        // self.handler = Some(handler);
        // self.func = Some(func.clone());

        // object
        //     .set_value_by_key("myFunc", func)
        //     .unwrap();

        let object = context.get_global().unwrap();

        let function =
            V8Value::create_function("sendMessage", V8Handler::new(SendMessageHandler::new()))
                .unwrap();

        object
            .set_value_by_key("sendMessage", function)
            .unwrap();
    }

    fn on_context_released(&mut self, _browser: Browser, _frame: Frame, _context: V8Context) {
        // Clear our references when the context is released
        self.handler = None;
        self.func = None;
    }
}

pub struct SendMessageHandler {}

impl SendMessageHandler {
    pub fn new() -> Self {
        Self {}
    }
}

impl Drop for SendMessageHandler {
    fn drop(&mut self) {
        println!("SendMessageHandler dropped");
    }
}

impl V8HandlerCallbacks for SendMessageHandler {
    fn execute(
        &mut self,
        _: String,
        _: V8Value,
        _: usize,
        _arguments: Vec<V8Value>
    ) -> Result<i32> {
        println!("sendMessage called from render process");
        Ok(1)
    }
}
