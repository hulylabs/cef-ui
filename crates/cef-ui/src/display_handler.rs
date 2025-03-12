use crate::{
    Browser, CefString, CursorInfo, CursorType, Frame, LogSeverity, RefCountedPtr, Size, Wrappable,
    Wrapped, ref_counted_ptr
};
use cef_ui_sys::{
    cef_browser_t, cef_cursor_info_t, cef_cursor_type_t, cef_display_handler_t, cef_frame_t,
    cef_log_severity_t, cef_size_t, cef_string_list_t, cef_string_t
};
use std::{ffi::c_int, mem::zeroed};

pub trait DisplayHandlerCallbacks: Send + Sync + 'static {
    /// Called when a frame's address has changed.
    fn on_address_change(&mut self, browser: Browser, frame: Frame, url: &str);

    /// Called when the page title changes.
    fn on_title_change(&mut self, browser: Browser, title: &str);

    /// Called when the page icon changes.
    fn on_favicon_urlchange(&mut self, browser: Browser, icon_urls: Vec<String>);

    /// Called when web content in the page has toggled fullscreen mode. If
    /// |fullscreen| is true (1) the content will automatically be sized to fill
    /// the browser content area. If |fullscreen| is false (0) the content will
    /// automatically return to its original size and position. With Alloy style
    /// the client is responsible for triggering the fullscreen transition (for
    /// example, by calling cef_window_t::SetFullscreen when using Views). With
    /// Chrome style the fullscreen transition will be triggered automatically.
    /// The cef_window_delegate_t::OnWindowFullscreenTransition function will be
    /// called during the fullscreen transition for notification purposes.
    fn on_fullscreen_mode_change(&mut self, browser: Browser, fullscreen: bool);

    /// Called when the browser is about to display a tooltip. |text| contains the
    /// text that will be displayed in the tooltip. To handle the display of the
    /// tooltip yourself return true (1). Otherwise, you can optionally modify
    /// |text| and then return false (0) to allow the browser to display the
    /// tooltip. When window rendering is disabled the application is responsible
    /// for drawing tooltips and the return value is ignored.
    fn on_tooltip(&mut self, browser: Browser, text: Option<String>) -> bool;

    /// Called when the browser receives a status message. |value| contains the
    /// text that will be displayed in the status message.
    fn on_status_message(&mut self, browser: Browser, value: Option<String>);

    /// Called to display a console message. Return true (1) to stop the message
    /// from being output to the console.
    fn on_console_message(
        &mut self,
        browser: Browser,
        level: LogSeverity,
        message: Option<String>,
        source: Option<String>,
        line: i32
    ) -> bool;

    /// Called when auto-resize is enabled via
    /// cef_browser_host_t::SetAutoResizeEnabled and the contents have auto-
    /// resized. |new_size| will be the desired size in view coordinates. Return
    /// true (1) if the resize was handled or false (0) for default handling.
    fn on_auto_resize(&mut self, browser: Browser, new_size: &Size) -> bool;

    /// Called when the overall page loading progress has changed. |progress|
    /// ranges from 0.0 to 1.0.
    fn on_loading_progress_change(&mut self, browser: Browser, progress: f64);

    /// Called when the browser's cursor has changed. If |type| is CT_CUSTOM then
    /// |custom_cursor_info| will be populated with the custom cursor information.
    /// Return true (1) if the cursor change was handled or false (0) for default
    /// handling.
    fn on_cursor_change(
        &mut self,
        browser: Browser,
        cursor: u64,
        cursor_type: CursorType,
        custom_cursor_info: Option<CursorInfo>
    ) -> bool;

    /// Called when the browser's access to an audio and/or video source has
    /// changed.
    fn on_media_access_change(&mut self, browser: Browser, has_video: bool, has_audio: bool);
}

ref_counted_ptr!(DisplayHandler, cef_display_handler_t);

impl DisplayHandler {
    pub fn new<C: DisplayHandlerCallbacks>(delegate: C) -> Self {
        Self(DisplayHandlerWrapper::new(delegate).wrap())
    }
}

struct DisplayHandlerWrapper(Box<dyn DisplayHandlerCallbacks>);

impl DisplayHandlerWrapper {
    pub fn new<C: DisplayHandlerCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }

    /// Called when a frame's address has changed.
    unsafe extern "C" fn c_on_address_change(
        this: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        url: *const cef_string_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let frame = Frame::from_ptr_unchecked(frame);
        let url: String = CefString::from_ptr_unchecked(url).into();

        this.0
            .on_address_change(browser, frame, &url);
    }

    /// Called when the page title changes.
    unsafe extern "C" fn c_on_title_change(
        this: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        title: *const cef_string_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let title: String = CefString::from_ptr_unchecked(title).into();

        this.0
            .on_title_change(browser, &title);
    }

    /// Called when the page icon changes.
    unsafe extern "C" fn c_on_favicon_urlchange(
        this: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        _icon_urls: cef_string_list_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        // TODO: For some reason this line crashes CEF (Trace/breakpoint trap (core dumped))
        // let icon_urls = CefStringList::from_ptr(icon_urls).map_or(Vec::new(), |s| s.into());
        let icon_urls = Vec::new();

        this.0
            .on_favicon_urlchange(browser, icon_urls);
    }

    /// Called when web content in the page has toggled fullscreen mode. If
    /// |fullscreen| is true (1) the content will automatically be sized to fill
    /// the browser content area. If |fullscreen| is false (0) the content will
    /// automatically return to its original size and position. With Alloy style
    /// the client is responsible for triggering the fullscreen transition (for
    /// example, by calling cef_window_t::SetFullscreen when using Views). With
    /// Chrome style the fullscreen transition will be triggered automatically.
    /// The cef_window_delegate_t::OnWindowFullscreenTransition function will be
    /// called during the fullscreen transition for notification purposes.
    unsafe extern "C" fn c_on_fullscreen_mode_change(
        this: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        fullscreen: c_int
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);

        this.0
            .on_fullscreen_mode_change(browser, fullscreen != 0);
    }

    /// Called when the browser is about to display a tooltip. |text| contains the
    /// text that will be displayed in the tooltip. To handle the display of the
    /// tooltip yourself return true (1). Otherwise, you can optionally modify
    /// |text| and then return false (0) to allow the browser to display the
    /// tooltip. When window rendering is disabled the application is responsible
    /// for drawing tooltips and the return value is ignored.
    unsafe extern "C" fn c_on_tooltip(
        this: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        text: *mut cef_string_t
    ) -> ::std::os::raw::c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let text: Option<String> = CefString::from_ptr(text).map(|s| s.into());

        this.0.on_tooltip(browser, text) as ::std::os::raw::c_int
    }

    /// Called when the browser receives a status message. |value| contains the
    /// text that will be displayed in the status message.
    unsafe extern "C" fn c_on_status_message(
        this: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        value: *const cef_string_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let value: Option<String> = CefString::from_ptr(value).map(|s| s.into());

        this.0
            .on_status_message(browser, value);
    }

    /// Called to display a console message. Return true (1) to stop the message
    /// from being output to the console.
    unsafe extern "C" fn c_on_console_message(
        this: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        level: cef_log_severity_t,
        message: *const cef_string_t,
        source: *const cef_string_t,
        line: ::std::os::raw::c_int
    ) -> ::std::os::raw::c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let message: Option<String> = CefString::from_ptr(message).map(|s| s.into());
        let source: Option<String> = CefString::from_ptr(source).map(|s| s.into());

        this.0
            .on_console_message(browser, level.into(), message, source, line)
            as ::std::os::raw::c_int
    }

    /// Called when auto-resize is enabled via
    /// cef_browser_host_t::SetAutoResizeEnabled and the contents have auto-
    /// resized. |new_size| will be the desired size in view coordinates. Return
    /// true (1) if the resize was handled or false (0) for default handling.
    unsafe extern "C" fn c_on_auto_resize(
        this: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        new_size: *const cef_size_t
    ) -> ::std::os::raw::c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);

        let new_size: Size = (*new_size).into();
        this.0
            .on_auto_resize(browser, &new_size) as ::std::os::raw::c_int
    }

    /// Called when the overall page loading progress has changed. |progress|
    /// ranges from 0.0 to 1.0.
    unsafe extern "C" fn c_on_loading_progress_change(
        this: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        progress: f64
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);

        this.0
            .on_loading_progress_change(browser, progress);
    }

    /// Called when the browser's cursor has changed. If |type| is CT_CUSTOM then
    /// |custom_cursor_info| will be populated with the custom cursor information.
    /// Return true (1) if the cursor change was handled or false (0) for default
    /// handling.
    unsafe extern "C" fn c_on_cursor_change(
        this: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        cursor: ::std::os::raw::c_ulong,
        cursor_type: cef_cursor_type_t,
        custom_cursor_info: *const cef_cursor_info_t
    ) -> ::std::os::raw::c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let cursor_type: CursorType = cursor_type.into();
        let custom_cursor_info: Option<CursorInfo> = if cursor_type == CursorType::Custom {
            Some((*custom_cursor_info).into())
        } else {
            None
        };

        this.0
            .on_cursor_change(browser, cursor, cursor_type, custom_cursor_info)
            as ::std::os::raw::c_int
    }

    /// Called when the browser's access to an audio and/or video source has
    /// changed.
    unsafe extern "C" fn c_on_media_access_change(
        this: *mut cef_display_handler_t,
        browser: *mut cef_browser_t,
        has_video_access: ::std::os::raw::c_int,
        has_audio_access: ::std::os::raw::c_int
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);

        this.0
            .on_media_access_change(browser, has_video_access != 0, has_audio_access != 0);
    }
}

impl Wrappable for DisplayHandlerWrapper {
    type Cef = cef_display_handler_t;

    fn wrap(self) -> RefCountedPtr<cef_display_handler_t> {
        RefCountedPtr::wrap(
            cef_display_handler_t {
                base:                       unsafe { zeroed() },
                on_address_change:          Some(Self::c_on_address_change),
                on_title_change:            Some(Self::c_on_title_change),
                on_favicon_urlchange:       Some(Self::c_on_favicon_urlchange),
                on_fullscreen_mode_change:  Some(Self::c_on_fullscreen_mode_change),
                on_tooltip:                 Some(Self::c_on_tooltip),
                on_status_message:          Some(Self::c_on_status_message),
                on_console_message:         Some(Self::c_on_console_message),
                on_auto_resize:             Some(Self::c_on_auto_resize),
                on_loading_progress_change: Some(Self::c_on_loading_progress_change),
                on_cursor_change:           Some(Self::c_on_cursor_change),
                on_media_access_change:     Some(Self::c_on_media_access_change)
            },
            self
        )
    }
}
