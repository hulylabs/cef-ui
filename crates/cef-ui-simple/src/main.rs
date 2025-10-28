use anyhow::Result;
use cef_ui::{
    App, AppCallbacks, Browser, BrowserHost, BrowserSettings, Client, ClientCallbacks, CommandLine,
    Context, ContextMenuHandler, ContextMenuHandlerCallbacks, ContextMenuParams, Frame,
    LifeSpanHandler, LifeSpanHandlerCallbacks, LogSeverity, MainArgs, MenuModel, Settings,
    WindowInfo
};
use cef_ui_sys::cef_quit_message_loop;
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
        // Prevent popups from spawning.
        if let Err(e) = model.clear() {
            error!("{}", e);
        }
    }
}

pub struct MyLifeSpanHandlerCallbacks;

#[allow(unused_variables)]
impl LifeSpanHandlerCallbacks for MyLifeSpanHandlerCallbacks {
    fn on_before_close(&mut self, browser: Browser) {
        unsafe {
            cef_quit_message_loop();
        }
    }
}

pub struct MyClientCallbacks;

impl ClientCallbacks for MyClientCallbacks {
    fn get_context_menu_handler(&mut self) -> Option<ContextMenuHandler> {
        Some(ContextMenuHandler::new(MyContextMenuHandler {}))
    }

    fn get_life_span_handler(&mut self) -> Option<LifeSpanHandler> {
        Some(LifeSpanHandler::new(MyLifeSpanHandlerCallbacks {}))
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
        info!("Setting CEF command line switches.");

        // This is to disable scary warnings on macOS.
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
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
    // This routes log macros through tracing.
    LogTracer::init()?;

    // Setup the tracing subscriber globally.
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::from_level(Level::DEBUG))
        .finish();

    set_global_default(subscriber)?;

    // Ensure the root cache directory exists.
    let root_cache_dir = get_root_cache_dir()?;

    // The command line arguments.
    let main_args = MainArgs::new()?;

    // Prepare the outermost CEF settings. We will drive the
    // event loop ourselves and use offscreen rendering.
    let settings = Settings::new()
        .log_severity(LogSeverity::Info)
        .root_cache_path(&root_cache_dir)?
        .no_sandbox(false);

    // Create the outermost CEF application.
    let app = App::new(MyAppCallbacks {});

    // Create the CEF context which is the outermost way we interact
    // with CEF, mainly for booting it up and shutting it down.
    let context = Context::new(main_args, settings, Some(app));

    // If this is a CEF subprocess, let it run and then
    // emit the proper exit code so CEF can clean up.
    if let Some(code) = context.is_cef_subprocess() {
        exit(code);
    }

    // Initialize CEF.
    context.initialize()?;

    // Create the window.
    let window_info = WindowInfo::new()
        .window_name(&String::from("cef-ui-simple"))
        .runtime_style(cef_ui::RuntimeStyle::Alloy);

    // Create the browser settings.
    let browser_settings = BrowserSettings::new();

    // The browser-specific client.
    let client = Client::new(MyClientCallbacks);

    // Create a new browser.
    BrowserHost::create_browser_sync(
        &window_info,
        client,
        "https://doc.rust-lang.org/book/",
        &browser_settings,
        None,
        None
    );

    info!("Running CEF message loop.");

    // Run the message loop.
    context.run_message_loop();

    info!("Shutting down CEF.");

    // Shutdown CEF.
    context.shutdown();

    Ok(())
}

/// Ensure the root cache directory exists.
pub fn get_root_cache_dir() -> Result<PathBuf> {
    let path = std::env::temp_dir().join("cef-ui-simple");
    if !path.exists() {
        create_dir_all(&path)?;
    }

    Ok(path)
}
