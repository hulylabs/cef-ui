use crate::{
    frame, ref_counted_ptr, Browser, CefString, Client, CommandLine, DictionaryValue, Frame, RefCountedPtr, Value, Wrappable, Wrapped
};
use cef_ui_sys::{
    cef_render_process_handler_t, cef_client_t, cef_command_line_t, cef_preference_registrar_t,
    cef_preferences_type_t, cef_string_t
};
use std::{ffi::c_int, mem::zeroed, ptr::null_mut};

/// Structure used to implement render process callbacks. The functions of this
/// structure will be called on the render process main thread (TID_RENDERER)
/// unless otherwise indicated.
pub trait RenderProcessHandlerCallbacks: Send + Sync + 'static {
    fn on_browser_created(
        &mut self,
        browser: Browser,
        extra_info: Option<DictionaryValue>
    );

    fn on_browser_destroyed(
        &mut self,
        browser: Browser
    );
}

// Structure used to implement render process callbacks. The functions of this
// structure will be called on the render process main thread (TID_RENDERER)
// unless otherwise indicated.
ref_counted_ptr!(RenderProcessHandler, cef_render_process_handler_t);

impl RenderProcessHandler {
    pub fn new<C: RenderProcessHandlerCallbacks>(delegate: C) -> Self {
        Self(RenderProcessHandlerWrapper::new(delegate).wrap())
    }
}

/// Translates CEF -> Rust callbacks.
struct RenderProcessHandlerWrapper(Box<dyn RenderProcessHandlerCallbacks>);

impl RenderProcessHandlerWrapper {
    pub fn new<C: RenderProcessHandlerCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }
    unsafe extern "C" fn с_on_browser_created(
        self_: *mut _cef_render_process_handler_t,
        browser: *mut _cef_browser_t,
        extra_info: *mut _cef_dictionary_value_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let extra_info = DictionaryValue::from_ptr(extra_info);

        this.0
            .on_browser_created(browser, extra_info);
    }

}

impl Wrappable for RenderProcessHandlerWrapper {
    type Cef = cef_browser_process_handler_t;

    /// Converts this to a smart pointer.
    fn wrap(self) -> RefCountedPtr<cef_render_process_handler_t> {
        RefCountedPtr::wrap(
            cef_render_process_handler_t {
                base:                           unsafe { zeroed() },
                on_web_kit_initialized:         None,
                on_browser_created:             Some(Self::с_on_browser_created),
                on_browser_destroyed:           None,
                get_load_handler:               None,
                on_context_created:             None,
                on_context_released:            None,
                on_uncaught_exception:          None,
                on_focused_node_changed:        None,
                on_process_message_received:    None,
            },
            self
        )
    }
}
