use anyhow::Result;
use cef_ui::{
    App, AppCallbacks, BeforeDownloadCallback, Browser, BrowserHost, BrowserProcessHandler,
    BrowserSettings, Client, ClientCallbacks, CommandLine, Context, ContextMenuHandler,
    ContextMenuHandlerCallbacks, ContextMenuParams, DictionaryValue, DownloadHandler,
    DownloadHandlerCallbacks, DownloadItem, DownloadItemCallback, EventFlags, Frame,
    KeyboardHandler, LifeSpanHandler, LifeSpanHandlerCallbacks, LogSeverity, MainArgs,
    MenuCommandId, MenuModel, Point, PopupFeatures, QuickMenuEditStateFlags, RenderHandler,
    RunContextMenuCallback, RunQuickMenuCallback, Settings, Size, WindowInfo,
    WindowOpenDisposition
};
use cef_ui_sys::cef_quit_message_loop;
use std::{fs::create_dir_all, path::PathBuf, process::exit};
use tracing::{Level, error, info, level_filters::LevelFilter, subscriber::set_global_default};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

/// Context menu callbacks.
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

    fn run_context_menu(
        &mut self,
        browser: Browser,
        frame: Frame,
        params: ContextMenuParams,
        model: MenuModel,
        callback: RunContextMenuCallback
    ) -> bool {
        false
    }

    fn on_context_menu_command(
        &mut self,
        browser: Browser,
        frame: Frame,
        params: ContextMenuParams,
        command_id: MenuCommandId,
        event_flags: EventFlags
    ) -> bool {
        false
    }

    fn on_context_menu_dismissed(&mut self, browser: Browser, frame: Frame) {}

    fn run_quick_menu(
        &mut self,
        browser: Browser,
        frame: Frame,
        location: &Point,
        size: &Size,
        edit_state_flags: QuickMenuEditStateFlags,
        callback: RunQuickMenuCallback
    ) -> bool {
        false
    }

    fn on_quick_menu_command(
        &mut self,
        browser: Browser,
        frame: Frame,
        command_id: MenuCommandId,
        event_flags: EventFlags
    ) -> bool {
        false
    }

    fn on_quick_menu_dismissed(&mut self, browser: Browser, frame: Frame) {}
}

/// Life span callbacks.
pub struct MyLifeSpanHandlerCallbacks;

#[allow(unused_variables)]
impl LifeSpanHandlerCallbacks for MyLifeSpanHandlerCallbacks {
    unsafe fn on_before_popup(
        &mut self,
        browser: Browser,
        frame: Frame,
        popup_id: i32,
        target_url: Option<String>,
        target_frame_name: Option<String>,
        target_disposition: WindowOpenDisposition,
        user_gesture: bool,
        popup_features: PopupFeatures,
        window_info: &mut WindowInfo,
        client: &mut Option<Client>,
        settings: &mut BrowserSettings,
        extra_info: &mut Option<DictionaryValue>,
        no_javascript_access: &mut bool
    ) -> bool {
        true
    }

    fn on_before_dev_tools_popup(
        &mut self,
        browser: Browser,
        window_info: &mut WindowInfo,
        client: &mut Option<Client>,
        settings: &mut BrowserSettings,
        extra_info: &mut Option<DictionaryValue>,
        use_default_window: &mut bool
    ) {
    }

    fn on_after_created(&mut self, browser: Browser) {}

    fn do_close(&mut self, browser: Browser) -> bool {
        false
    }

    fn on_before_close(&mut self, browser: Browser) {
        // If you have more than one browser open, you want to only
        // call this when the number of open browsers reaches zero.
        unsafe {
            cef_quit_message_loop();
        }
    }
}

pub struct MyDownloadHandlerCallbacks;
impl DownloadHandlerCallbacks for MyDownloadHandlerCallbacks {
    fn on_before_download(
        &mut self,
        _browser: Browser,
        _download_item: DownloadItem,
        suggested_name: &str,
        callback: BeforeDownloadCallback
    ) -> bool {
        info!("Starting download of {}", suggested_name);
        let download_dir = dirs::download_dir().expect("failed to get download directory");
        let download_path = download_dir.join(suggested_name);
        let download_path = download_path
            .to_str()
            .expect("failed to convert path to str");
        _ = callback.continue_download(Some(download_path), false);
        true
    }

    fn on_download_updated(
        &mut self,
        _: Browser,
        download_item: DownloadItem,
        _: DownloadItemCallback
    ) {
        let path = PathBuf::from(
            download_item
                .get_full_path()
                .unwrap()
        );

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        // let name = "a";
        let received = download_item
            .get_received_bytes()
            .unwrap();
        let total = download_item
            .get_total_bytes()
            .unwrap();
        let speed = download_item
            .get_current_speed()
            .unwrap();

        if speed == 0 {
            info!("Downloading {}: {}/{}", name, received, total);
            return;
        }
        let remaining_time = (total - received) / speed;
        info!(
            "Downloading {}: {}/{}. Remaining time: {} seconds",
            name, received, total, remaining_time
        );
    }

    fn can_download(&mut self, _browser: Browser, _url: &str, _request_method: &str) -> bool {
        true
    }
}

/// Client callbacks.
pub struct MyClientCallbacks;

impl ClientCallbacks for MyClientCallbacks {
    fn get_context_menu_handler(&mut self) -> Option<ContextMenuHandler> {
        Some(ContextMenuHandler::new(MyContextMenuHandler {}))
    }

    fn get_keyboard_handler(&mut self) -> Option<KeyboardHandler> {
        None
    }

    fn get_life_span_handler(&mut self) -> Option<LifeSpanHandler> {
        Some(LifeSpanHandler::new(MyLifeSpanHandlerCallbacks {}))
    }

    fn get_render_handler(&mut self) -> Option<RenderHandler> {
        None
    }

    fn get_display_handler(&mut self) -> Option<cef_ui::DisplayHandler> {
        None
    }

    fn get_load_handler(&mut self) -> Option<cef_ui::LoadHandler> {
        None
    }

    fn get_download_handler(&mut self) -> Option<cef_ui::DownloadHandler> {
        Some(DownloadHandler::new(MyDownloadHandlerCallbacks {}))
    }

    fn on_process_message_received(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _source_process: cef_ui::ProcessId,
        _message: cef_ui::ProcessMessage
    ) -> bool {
        true
    }

    fn get_request_handler(&mut self) -> Option<cef_ui::RequestHandler> {
        None
    }
}

/// Application callbacks.
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

    fn get_browser_process_handler(&mut self) -> Option<BrowserProcessHandler> {
        None
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
        "https://github.com/hulylabs/cef-ui/releases/tag/v132",
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

// TODO: Make this platform-specific!

/// Ensure the root cache directory exists.
pub fn get_root_cache_dir() -> Result<PathBuf> {
    let path = PathBuf::from("/tmp/cef-ui-simple");
    if !path.exists() {
        create_dir_all(&path)?;
    }

    Ok(path)
}
