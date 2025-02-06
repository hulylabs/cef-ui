use crate::{
    ref_counted_ptr, v8_context::V8Context, Browser, DictionaryValue, Frame, RefCountedPtr, Wrappable, Wrapped
};
use cef_ui_sys::{
    cef_browser_t, cef_dictionary_value_t, cef_frame_t, cef_render_process_handler_t, cef_v8context_t
};
use std::mem::zeroed;

/// Structure used to implement render process callbacks. The functions of this
/// structure will be called on the render process main thread (TID_RENDERER)
/// unless otherwise indicated.
pub trait RenderProcessHandlerCallbacks: Send + Sync + 'static {
    /// Called after WebKit has been initialized.
    fn on_web_kit_initialized(
        &mut self
    );

    /// Called after a browser has been created. When browsing cross-origin a new
    /// browser will be created before the old browser with the same identifier is
    /// destroyed. |extra_info| is an optional read-only value originating from
    /// cef_browser_host_t::cef_browser_host_create_browser(),
    /// cef_browser_host_t::cef_browser_host_create_browser_sync(),
    /// cef_life_span_handler_t::on_before_popup() or
    /// cef_browser_view_t::cef_browser_view_create().
    fn on_browser_created(
        &mut self,
        browser: Browser,
        extra_info: Option<DictionaryValue>
    );

    /// Called before a browser is destroyed.
    fn on_browser_destroyed(
        &mut self,
        browser: Browser
    );

    // /// Return the handler for browser load status events.
    // struct _cef_load_handler_t*(CEF_CALLBACK* get_load_handler)(
    //     struct _cef_render_process_handler_t* self);

    /// Called immediately after the V8 context for a frame has been created. To
    /// retrieve the JavaScript 'window' object use the
    /// cef_v8context_t::get_global() function. V8 handles can only be accessed
    /// from the thread on which they are created. A task runner for posting tasks
    /// on the associated thread can be retrieved via the
    /// cef_v8context_t::get_task_runner() function.
    fn on_context_created(
        &mut self,
        browser: Browser,
        frame: Frame,
        context: V8Context
    );

    // /// Called immediately before the V8 context for a frame is released. No
    // /// references to the context should be kept after this function is called.
    // void(CEF_CALLBACK* on_context_released)(
    //     struct _cef_render_process_handler_t* self,
    //     struct _cef_browser_t* browser,
    //     struct _cef_frame_t* frame,
    //     struct _cef_v8context_t* context);

    // /// Called for global uncaught exceptions in a frame. Execution of this
    // /// callback is disabled by default. To enable set
    // /// cef_settings_t.uncaught_exception_stack_size > 0.
    // void(CEF_CALLBACK* on_uncaught_exception)(
    //     struct _cef_render_process_handler_t* self,
    //     struct _cef_browser_t* browser,
    //     struct _cef_frame_t* frame,
    //     struct _cef_v8context_t* context,
    //     struct _cef_v8exception_t* exception,
    //     struct _cef_v8stack_trace_t* stackTrace);

    // /// Called when a new node in the the browser gets focus. The |node| value may
    // /// be NULL if no specific node has gained focus. The node object passed to
    // /// this function represents a snapshot of the DOM at the time this function
    // /// is executed. DOM objects are only valid for the scope of this function. Do
    // /// not keep references to or attempt to access any DOM objects outside the
    // /// scope of this function.
    // void(CEF_CALLBACK* on_focused_node_changed)(
    //     struct _cef_render_process_handler_t* self,
    //     struct _cef_browser_t* browser,
    //     struct _cef_frame_t* frame,
    //     struct _cef_domnode_t* node);

    // /// Called when a new message is received from a different process. Return
    // /// true (1) if the message was handled or false (0) otherwise. It is safe to
    // /// keep a reference to |message| outside of this callback.
    // int(CEF_CALLBACK* on_process_message_received)(
    //     struct _cef_render_process_handler_t* self,
    //     struct _cef_browser_t* browser,
    //     struct _cef_frame_t* frame,
    //     cef_process_id_t source_process,
    //     struct _cef_process_message_t* message);
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

    unsafe extern "C" fn c_on_browser_created(
        this: *mut cef_render_process_handler_t,
        browser: *mut cef_browser_t,
        extra_info: *mut cef_dictionary_value_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let extra_info = DictionaryValue::from_ptr(extra_info);

        this.0
            .on_browser_created(browser, extra_info);
    }

    unsafe extern "C" fn on_web_kit_initialized(this: *mut cef_render_process_handler_t) {
        let this: &mut Self = Wrapped::wrappable(this);

        this.0
            .on_web_kit_initialized();
    }

    unsafe extern "C" fn on_context_created(
        this : *mut cef_render_process_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        context: *mut cef_v8context_t
    ) {
        let this : &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let frame = Frame::from_ptr_unchecked(frame);
        let context = V8Context::from_ptr_unchecked(context);

        this.0
            .on_context_created(browser, frame, context);
    }
}

impl Wrappable for RenderProcessHandlerWrapper {
    type Cef = cef_render_process_handler_t;

    /// Converts this to a smart pointer.
    fn wrap(self) -> RefCountedPtr<cef_render_process_handler_t> {
        RefCountedPtr::wrap(
            cef_render_process_handler_t {
                base:                           unsafe { zeroed() },
                on_web_kit_initialized:         Some(Self::on_web_kit_initialized),
                on_browser_created:             Some(Self::c_on_browser_created),
                on_browser_destroyed:           None,
                get_load_handler:               None,
                on_context_created:             Some(Self::on_context_created),
                on_context_released:            None,
                on_uncaught_exception:          None,
                on_focused_node_changed:        None,
                on_process_message_received:    None,
            },
            self
        )
    }
}
