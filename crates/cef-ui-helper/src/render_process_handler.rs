use crate::{
    Browser, RefCountedPtr, Wrappable, Wrapped, frame::Frame, ref_counted_ptr,
    v8_context::V8Context
};
use cef_ui_sys::{cef_browser_t, cef_frame_t, cef_render_process_handler_t, cef_v8context_t};
use parking_lot::Mutex;
use std::mem::zeroed;

pub trait RenderProcessHandlerCallbacks: Send + Sync + 'static {
    fn on_web_kit_initialized(&mut self) {}

    fn on_context_created(&mut self, _browser: Browser, _frame: Frame, _context: V8Context) {}
}

ref_counted_ptr!(RenderProcessHandler, cef_render_process_handler_t);

impl RenderProcessHandler {
    pub fn new<C: RenderProcessHandlerCallbacks>(delegate: C) -> Self {
        Self(RenderProcessHandlerWrapper::new(delegate).wrap())
    }
}

struct RenderProcessHandlerWrapper(Mutex<Box<dyn RenderProcessHandlerCallbacks>>);

impl RenderProcessHandlerWrapper {
    pub fn new<C: RenderProcessHandlerCallbacks>(delegate: C) -> Self {
        Self(Mutex::new(Box::new(delegate)))
    }

    unsafe extern "C" fn c_on_web_kit_initialized(this: *mut cef_render_process_handler_t) {
        let this: &mut Self = Wrapped::wrappable(this);
        this.0
            .lock()
            .on_web_kit_initialized();
    }

    unsafe extern "C" fn c_on_context_created(
        this: *mut cef_render_process_handler_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        context: *mut cef_v8context_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let frame = Frame::from_ptr_unchecked(frame);
        let context = V8Context::from_ptr_unchecked(context);

        this.0
            .lock()
            .on_context_created(browser, frame, context);
    }
}

impl Wrappable for RenderProcessHandlerWrapper {
    type Cef = cef_render_process_handler_t;

    fn wrap(self) -> RefCountedPtr<cef_render_process_handler_t> {
        RefCountedPtr::wrap(
            cef_render_process_handler_t {
                base:                        unsafe { zeroed() },
                on_web_kit_initialized:      Some(Self::c_on_web_kit_initialized),
                on_browser_created:          None,
                on_browser_destroyed:        None,
                get_load_handler:            None,
                on_context_created:          Some(Self::c_on_context_created),
                on_context_released:         None,
                on_uncaught_exception:       None,
                on_focused_node_changed:     None,
                on_process_message_received: None
            },
            self
        )
    }
}
