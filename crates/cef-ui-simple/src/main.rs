use anyhow::Result;
use cef_ui::{
    App, AppCallbacks, Browser, BrowserHost, BrowserSettings, Client, ClientCallbacks, CommandLine,
    Context, ContextMenuHandler, ContextMenuHandlerCallbacks, ContextMenuParams, Frame,
    LifeSpanHandler, LifeSpanHandlerCallbacks, LogSeverity, MainArgs, MenuModel, Settings,
    WindowInfo
};
use cef_ui_sys::{CEF_API_VERSION_13800, cef_api_hash, cef_quit_message_loop};
use std::{fs::create_dir_all, path::PathBuf, process::exit};
use tracing::{Level, error, info, level_filters::LevelFilter, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

pub struct MyContextMenuHandler;

#[allow(unused_variables)]
impl ContextMenuHandlerCallbacks for MyContextMenuHandler {
    fn on_before_context_menu(
        &mut self,
        browser: Browser,
        frame: Frame,
        params: ContextMenuParams,
        model: MenuModel
    ) {
        if let Err(e) = model.clear() {
            error!("{}", e);
        }
    }
}

pub struct MyLifeSpanHandlerCallbacks;
impl LifeSpanHandlerCallbacks for MyLifeSpanHandlerCallbacks {
    fn on_before_close(&mut self, _browser: Browser) {
        unsafe {
            cef_quit_message_loop();
        }
    }
}

struct MyRequestHandler;
impl cef_ui::RequestHandlerCallbacks for MyRequestHandler {
    fn on_render_process_terminated(
        &mut self,
        _browser: Browser,
        _status: cef_ui::TerminationStatus,
        _error_code: i32,
        _error_string: Option<String>
    ) {
        info!("Render process terminated, exiting application.");
    }
}

/// Client callbacks.
pub struct MyClientCallbacks;

impl ClientCallbacks for MyClientCallbacks {
    fn get_context_menu_handler(&mut self) -> Option<ContextMenuHandler> {
        Some(ContextMenuHandler::new(MyContextMenuHandler {}))
    }

    fn get_life_span_handler(&mut self) -> Option<LifeSpanHandler> {
        Some(LifeSpanHandler::new(MyLifeSpanHandlerCallbacks {}))
    }

    fn get_request_handler(&mut self) -> Option<cef_ui::RequestHandler> {
        Some(cef_ui::RequestHandler::new(MyRequestHandler {}))
    }
}

pub struct MyAppCallbacks;

#[allow(unused_variables)]
impl AppCallbacks for MyAppCallbacks {
    fn on_before_command_line_processing(
        &mut self,
        process_type: Option<&str>,
        command_line: Option<CommandLine>
    ) {
        #[cfg(all(target_os = "macos"))]
        if let Some(command_line) = command_line {
            if process_type.is_none() {
                if let Err(e) = command_line.append_switch("--use-mock-keychain") {
                    error!("{}", e);
                }
            }
        }
    }

    fn get_render_process_handler(&mut self) -> Option<cef_ui::RenderProcessHandler> {
        None
    }
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {}", e);

        exit(1);
    }
}

fn try_main() -> Result<()> {
    LogTracer::init()?;

    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::from_level(Level::TRACE))
        .finish();

    set_global_default(subscriber)?;

    unsafe {
        cef_api_hash(CEF_API_VERSION_13800 as i32, 0);
    }

    let root_cache_dir = get_root_cache_dir()?;
    let main_args = MainArgs::new()?;

    let settings = Settings::new()
        .log_severity(LogSeverity::Verbose)
        .root_cache_path(&root_cache_dir)?
        .no_sandbox(false);

    let app = App::new(MyAppCallbacks {});
    let context = Context::new(main_args, settings, Some(app));

    if let Some(code) = context.is_cef_subprocess() {
        exit(code);
    }

    context.initialize()?;

    let window_info = WindowInfo::new()
        .window_name(&String::from("cef-ui-simple"))
        .runtime_style(cef_ui::RuntimeStyle::Alloy);

    let browser_settings = BrowserSettings::new();
    let client = Client::new(MyClientCallbacks);

    BrowserHost::create_browser_sync(
        &window_info,
        client,
        "https://doc.rust-lang.org/book/",
        &browser_settings,
        None,
        None
    );

    context.run_message_loop();
    context.shutdown();

    Ok(())
}

pub fn get_root_cache_dir() -> Result<PathBuf> {
    let path = std::env::temp_dir().join("cef-ui-simple");
    if !path.exists() {
        create_dir_all(&path)?;
    }

    Ok(path)
}
