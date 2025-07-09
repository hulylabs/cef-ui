use crate::{Browser, CefString, RefCountedPtr, Wrappable, Wrapped, ref_counted_ptr};
use cef_ui_sys::{cef_browser_t, cef_dev_tools_message_observer_t, cef_string_t};
use std::{
    ffi::{c_int, c_void},
    mem::zeroed,
    slice::from_raw_parts
};

/// Callback interface for CefBrowserHost::AddDevToolsMessageObserver. The
/// methods of this class will be called on the browser process UI thread.
pub trait DevToolsMessageObserverCallbacks: Send + Sync + 'static {
    /// Method that will be called on receipt of a DevTools protocol message.
    /// |browser| is the originating browser instance. |message| is a UTF8-encoded
    /// JSON dictionary representing either a method result or an event. |message|
    /// is only valid for the scope of this callback and should be copied if
    /// necessary. Return true if the message was handled or false if the message
    /// should be further processed and passed to the OnDevToolsMethodResult or
    /// OnDevToolsEvent methods as appropriate.
    ///
    /// Method result dictionaries include an "id" (int) value that identifies the
    /// orginating method call sent from CefBrowserHost::SendDevToolsMessage, and
    /// optionally either a "result" (dictionary) or "error" (dictionary) value.
    /// The "error" dictionary will contain "code" (int) and "message" (string)
    /// values. Event dictionaries include a "method" (string) value and
    /// optionally a "params" (dictionary) value. See the DevTools protocol
    /// documentation at https://chromedevtools.github.io/devtools-protocol/ for
    /// details of supported method calls and the expected "result" or "params"
    /// dictionary contents. JSON dictionaries can be parsed using the
    /// CefParseJSON function if desired, however be aware of performance
    /// considerations when parsing large messages (some of which may exceed 1MB
    /// in size).
    fn on_dev_tools_message(&mut self, browser: Browser, message: &[u8]) -> bool;

    /// Method that will be called after attempted execution of a DevTools
    /// protocol method. |browser| is the originating browser instance.
    /// |message_id| is the "id" value that identifies the originating method call
    /// message. If the method succeeded |success| will be true and |result| will
    /// be the UTF8-encoded JSON "result" dictionary value (which may be empty).
    /// If the method failed |success| will be false and |result| will be the
    /// UTF8-encoded JSON "error" dictionary value. |result| is only valid for the
    /// scope of this callback and should be copied if necessary. See the
    /// OnDevToolsMessage documentation for additional details on |result|
    /// contents.
    fn on_dev_tools_method_result(
        &mut self,
        browser: Browser,
        message_id: i32,
        success: bool,
        result: &[u8]
    );

    /// Method that will be called on receipt of a DevTools protocol event.
    /// |browser| is the originating browser instance. |method| is the "method"
    /// value. |params| is the UTF8-encoded JSON "params" dictionary value (which
    /// may be empty). |params| is only valid for the scope of this callback and
    /// should be copied if necessary. See the OnDevToolsMessage documentation for
    /// additional details on |params| contents.
    fn on_dev_tools_event(&mut self, browser: Browser, method: &str, params: &[u8]);

    /// Method that will be called when the DevTools agent has attached. |browser|
    /// is the originating browser instance. This will generally occur in response
    /// to the first message sent while the agent is detached.
    fn on_dev_tools_agent_attached(&mut self, browser: Browser);

    /// Method that will be called when the DevTools agent has detached. |browser|
    /// is the originating browser instance. Any method results that were pending
    /// before the agent became detached will not be delivered, and any active
    /// event subscriptions will be canceled.
    fn on_dev_tools_agent_detached(&mut self, browser: Browser);
}

// Structure used to observe DevTools protocol messages. The functions of this
// structure will be called on the browser process UI thread.
ref_counted_ptr!(DevToolsMessageObserver, cef_dev_tools_message_observer_t);

impl DevToolsMessageObserver {
    pub fn new<C: DevToolsMessageObserverCallbacks>(delegate: C) -> Self {
        Self(DevToolsMessageObserverWrapper::new(delegate).wrap())
    }
}

/// Translates CEF -> Rust callbacks.
struct DevToolsMessageObserverWrapper(Box<dyn DevToolsMessageObserverCallbacks>);

impl DevToolsMessageObserverWrapper {
    pub fn new<C: DevToolsMessageObserverCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }

    unsafe extern "C" fn c_on_dev_tools_message(
        this: *mut cef_dev_tools_message_observer_t,
        browser: *mut cef_browser_t,
        message: *const c_void,
        message_size: usize
    ) -> c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let message = from_raw_parts(message as *const u8, message_size);

        this.0
            .on_dev_tools_message(browser, message) as c_int
    }

    unsafe extern "C" fn c_on_dev_tools_method_result(
        this: *mut cef_dev_tools_message_observer_t,
        browser: *mut cef_browser_t,
        message_id: c_int,
        success: c_int,
        result: *const c_void,
        result_size: usize
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let result = from_raw_parts(result as *const u8, result_size);

        this.0
            .on_dev_tools_method_result(browser, message_id, success != 0, result);
    }

    unsafe extern "C" fn c_on_dev_tools_event(
        this: *mut cef_dev_tools_message_observer_t,
        browser: *mut cef_browser_t,
        method: *const cef_string_t,
        params: *const c_void,
        params_size: usize
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let method: String = CefString::from_ptr_unchecked(method).into();
        let params = from_raw_parts(params as *const u8, params_size);

        this.0
            .on_dev_tools_event(browser, &method, params);
    }

    unsafe extern "C" fn c_on_dev_tools_agent_attached(
        this: *mut cef_dev_tools_message_observer_t,
        browser: *mut cef_browser_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);

        this.0
            .on_dev_tools_agent_attached(browser);
    }

    unsafe extern "C" fn c_on_dev_tools_agent_detached(
        this: *mut cef_dev_tools_message_observer_t,
        browser: *mut cef_browser_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);

        this.0
            .on_dev_tools_agent_detached(browser);
    }
}

impl Wrappable for DevToolsMessageObserverWrapper {
    type Cef = cef_dev_tools_message_observer_t;

    fn wrap(self) -> RefCountedPtr<cef_dev_tools_message_observer_t> {
        RefCountedPtr::wrap(
            cef_dev_tools_message_observer_t {
                base:                        unsafe { zeroed() },
                on_dev_tools_message:        Some(Self::c_on_dev_tools_message),
                on_dev_tools_method_result:  Some(Self::c_on_dev_tools_method_result),
                on_dev_tools_event:          Some(Self::c_on_dev_tools_event),
                on_dev_tools_agent_attached: Some(Self::c_on_dev_tools_agent_attached),
                on_dev_tools_agent_detached: Some(Self::c_on_dev_tools_agent_detached)
            },
            self
        )
    }
}
