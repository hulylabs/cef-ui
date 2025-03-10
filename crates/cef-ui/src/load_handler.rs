use crate::{
    Browser, CefString, ErrorCode, Frame, RefCountedPtr, TransitionType, Wrappable, Wrapped,
    ref_counted_ptr
};
use cef_ui_sys::{
    cef_browser_t, cef_errorcode_t, cef_frame_t, cef_load_handler_t, cef_string_t,
    cef_transition_type_t
};
use std::ffi::c_int;

/// Implement this structure to handle events related to browser load status.
/// The functions of this structure will be called on the browser process UI
/// thread or render process main thread (TID_RENDERER).
pub trait LoadHandlerCallbacks: Send + Sync + 'static {
    /// Called when the loading state has changed. This callback will be executed
    /// twice -- once when loading is initiated either programmatically or by user
    /// action, and once when loading is terminated due to completion,
    /// cancellation of failure. It will be called before any calls to OnLoadStart
    /// and after all calls to OnLoadError and/or OnLoadEnd.
    fn on_loading_state_change(
        &mut self,
        browser: Browser,
        is_loading: bool,
        can_go_back: bool,
        can_go_forward: bool
    );

    /// Called after a navigation has been committed and before the browser begins
    /// loading contents in the frame. The |frame| value will never be NULL --
    /// call the is_main() function to check if this frame is the main frame.
    /// |transition_type| provides information about the source of the navigation
    /// and an accurate value is only available in the browser process. Multiple
    /// frames may be loading at the same time. Sub-frames may start or continue
    /// loading after the main frame load has ended. This function will not be
    /// called for same page navigations (fragments, history state, etc.) or for
    /// navigations that fail or are canceled before commit. For notification of
    /// overall browser load status use OnLoadingStateChange instead.
    fn on_load_start(&mut self, browser: Browser, frame: Frame, transition_type: TransitionType);

    /// Called when the browser is done loading a frame. The |frame| value will
    /// never be NULL -- call the is_main() function to check if this frame is the
    /// main frame. Multiple frames may be loading at the same time. Sub-frames
    /// may start or continue loading after the main frame load has ended. This
    /// function will not be called for same page navigations (fragments, history
    /// state, etc.) or for navigations that fail or are canceled before commit.
    /// For notification of overall browser load status use OnLoadingStateChange
    /// instead.
    fn on_load_end(&mut self, browser: Browser, frame: Frame, http_status_code: i32);

    /// Called when a navigation fails or is canceled. This function may be called
    /// by itself if before commit or in combination with OnLoadStart/OnLoadEnd if
    /// after commit. |errorCode| is the error code number, |errorText| is the
    /// error text and |failedUrl| is the URL that failed to load. See
    /// net\base\net_error_list.h for complete descriptions of the error codes.
    fn on_load_error(
        &mut self,
        browser: Browser,
        frame: Frame,
        error_code: ErrorCode,
        error_text: &str,
        failed_url: &str
    );
}

// Implement this structure to handle events related to browser load status.
// The functions of this structure will be called on the browser process UI
// thread or render process main thread (TID_RENDERER).
ref_counted_ptr!(LoadHandler, cef_load_handler_t);

impl LoadHandler {
    pub fn new<C: LoadHandlerCallbacks>(delegate: C) -> Self {
        Self(LoadHandlerWrapper::new(delegate).wrap())
    }
}

/// Translates CEF -> Rust callbacks.
struct LoadHandlerWrapper(Box<dyn LoadHandlerCallbacks>);

impl LoadHandlerWrapper {
    pub fn new<C: LoadHandlerCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }

    /// Called when the loading state has changed. This callback will be executed
    /// twice -- once when loading is initiated either programmatically or by user
    /// action, and once when loading is terminated due to completion,
    /// cancellation of failure. It will be called before any calls to OnLoadStart
    /// and after all calls to OnLoadError and/or OnLoadEnd.
    unsafe extern "C" fn c_on_loading_state_change(
        this: *mut cef_load_handler_t,
        browser: *mut cef_browser_t,
        is_loading: c_int,
        can_go_back: c_int,
        can_go_forward: c_int
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        this.0.on_loading_state_change(
            browser,
            is_loading != 0,
            can_go_back != 0,
            can_go_forward != 0
        );
    }

    /// Called after a navigation has been committed and before the browser begins
    /// loading contents in the frame. The |frame| value will never be NULL --
    /// call the is_main() function to check if this frame is the main frame.
    /// |transition_type| provides information about the source of the navigation
    /// and an accurate value is only available in the browser process. Multiple
    /// frames may be loading at the same time. Sub-frames may start or continue
    /// loading after the main frame load has ended. This function will not be
    /// called for same page navigations (fragments, history state, etc.) or for
    /// navigations that fail or are canceled before commit. For notification of
    /// overall browser load status use OnLoadingStateChange instead.
    unsafe extern "C" fn c_on_load_start(
        this: *mut cef_load_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        transition_type: cef_transition_type_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let frame = Frame::from_ptr_unchecked(frame);
        this.0
            .on_load_start(browser, frame, transition_type.into());
    }

    /// Called when the browser is done loading a frame. The |frame| value will
    /// never be NULL -- call the is_main() function to check if this frame is the
    /// main frame. Multiple frames may be loading at the same time. Sub-frames
    /// may start or continue loading after the main frame load has ended. This
    /// function will not be called for same page navigations (fragments, history
    /// state, etc.) or for navigations that fail or are canceled before commit.
    /// For notification of overall browser load status use OnLoadingStateChange
    /// instead.
    unsafe extern "C" fn c_on_load_end(
        this: *mut cef_load_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        http_status_code: c_int
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let frame = Frame::from_ptr_unchecked(frame);
        this.0
            .on_load_end(browser, frame, http_status_code);
    }

    /// Called when a navigation fails or is canceled. This function may be called
    /// by itself if before commit or in combination with OnLoadStart/OnLoadEnd if
    /// after commit. |errorCode| is the error code number, |errorText| is the
    /// error text and |failedUrl| is the URL that failed to load. See
    /// net\base\net_error_list.h for complete descriptions of the error codes.
    unsafe extern "C" fn c_on_load_error(
        this: *mut cef_load_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        error_code: cef_errorcode_t,
        error_text: *const cef_string_t,
        failed_url: *const cef_string_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let frame = Frame::from_ptr_unchecked(frame);
        let error_text: String = CefString::from_ptr_unchecked(error_text).into();
        let failed_url: String = CefString::from_ptr_unchecked(failed_url).into();

        this.0
            .on_load_error(browser, frame, error_code.into(), &error_text, &failed_url);
    }
}

impl Wrappable for LoadHandlerWrapper {
    type Cef = cef_load_handler_t;

    /// Converts this to a smart pointer.
    fn wrap(self) -> RefCountedPtr<cef_load_handler_t> {
        RefCountedPtr::wrap(
            cef_load_handler_t {
                base:                    unsafe { std::mem::zeroed() },
                on_loading_state_change: Some(Self::c_on_loading_state_change),
                on_load_start:           Some(Self::c_on_load_start),
                on_load_end:             Some(Self::c_on_load_end),
                on_load_error:           Some(Self::c_on_load_error)
            },
            self
        )
    }
}
