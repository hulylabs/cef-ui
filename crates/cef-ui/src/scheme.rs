use anyhow::Result;
use cef_ui_sys::{
    cef_browser_t, cef_frame_t, cef_register_scheme_handler_factory, cef_request_t,
    cef_resource_handler_t, cef_scheme_handler_factory_t, cef_scheme_registrar_t, cef_string_t
};

use crate::{
    Browser, CefString, Frame, RefCountedPtr, Request, ResourceHandler, Wrappable, Wrapped,
    ref_counted_ptr, try_c
};
use std::mem::zeroed;

// Class that manages custom scheme registrations.
ref_counted_ptr!(SchemeRegistrar, cef_scheme_registrar_t);

/// Class that manages custom scheme registrations.
impl SchemeRegistrar {
    /// Register a custom scheme. This method should not be called for the
    /// built-in HTTP, HTTPS, FILE, FTP, ABOUT and DATA schemes.
    ///
    /// See cef_scheme_options_t for possible values for |options|.
    ///
    /// This function may be called on any thread. It should only be called once
    /// per unique |scheme_name| value. If |scheme_name| is already registered or
    /// if an error occurs this method will return false.
    pub fn add_custom_scheme(&self, scheme_name: &str, options: i32) -> Result<bool> {
        try_c!(self, add_custom_scheme, {
            let scheme_name = CefString::new(scheme_name);
            Ok(add_custom_scheme(self.as_ptr(), scheme_name.as_ptr(), options) == 1)
        })
    }
}

pub trait SchemeHandlerFactoryCallbacks: Send + Sync + 'static {
    /// Return a new resource handler instance to handle the request or an empty
    /// reference to allow default handling of the request. |browser| and |frame|
    /// will be the browser window and frame respectively that originated the
    /// request or NULL if the request did not originate from a browser window
    /// (for example, if the request came from CefURLRequest). The |request|
    /// object passed to this method cannot be modified.
    fn create(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _scheme_name: &str,
        _request: Request
    ) -> Option<ResourceHandler> {
        None
    }
}

// Class that creates CefResourceHandler instances for handling scheme
// requests. The methods of this class will always be called on the IO thread.
ref_counted_ptr!(SchemeHandlerFactory, cef_scheme_handler_factory_t);

/// Class that creates CefResourceHandler instances for handling scheme
/// requests. The methods of this class will always be called on the IO thread.
impl SchemeHandlerFactory {
    pub fn new<C: SchemeHandlerFactoryCallbacks>(delegate: C) -> Self {
        Self(SchemeHandlerFactoryWrapper::new(delegate).wrap())
    }
}

struct SchemeHandlerFactoryWrapper(Box<dyn SchemeHandlerFactoryCallbacks>);

impl SchemeHandlerFactoryWrapper {
    pub fn new<C: SchemeHandlerFactoryCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }

    unsafe extern "C" fn c_create(
        this: *mut cef_scheme_handler_factory_t,
        browser: *mut cef_browser_t,
        frame: *mut cef_frame_t,
        scheme_name: *const cef_string_t,
        request: *mut cef_request_t
    ) -> *mut cef_resource_handler_t {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let frame = Frame::from_ptr_unchecked(frame);
        let scheme_name: String = CefString::from_ptr_unchecked(scheme_name).into();
        let request = Request::from_ptr_unchecked(request);

        match this
            .0
            .create(browser, frame, &scheme_name, request)
        {
            Some(handler) => handler.into_raw(),
            None => std::ptr::null_mut()
        }
    }
}

impl Wrappable for SchemeHandlerFactoryWrapper {
    type Cef = cef_scheme_handler_factory_t;

    fn wrap(self) -> RefCountedPtr<cef_scheme_handler_factory_t> {
        RefCountedPtr::wrap(
            cef_scheme_handler_factory_t {
                base:   unsafe { zeroed() },
                create: Some(Self::c_create)
            },
            self
        )
    }
}

/// Register a scheme handler factory with the global request context. An empty
/// |domain_name| value for a standard scheme will cause the factory to match
/// all domain names. The |domain_name| value will be ignored for non-standard
/// schemes. If |scheme_name| is a built-in scheme and no handler is returned by
/// |factory| then the built-in scheme handler factory will be called. If
/// |scheme_name| is a custom scheme then you must also implement the
/// CefApp::OnRegisterCustomSchemes() method in all processes. This function may
/// be called multiple times to change or remove the factory that matches the
/// specified |scheme_name| and optional |domain_name|. Returns false if an
/// error occurs. This function may be called on any thread in the browser
/// process. Using this function is equivalent to calling
/// CefRequestContext::GetGlobalContext()->RegisterSchemeHandlerFactory().
pub fn register_scheme_handler_factory(
    scheme_name: &str,
    domain_name: &str,
    factory: SchemeHandlerFactory
) -> bool {
    let scheme_name = CefString::new(scheme_name);
    let domain_name = CefString::new(domain_name);

    unsafe {
        cef_register_scheme_handler_factory(
            scheme_name.as_ptr(),
            domain_name.as_ptr(),
            factory.into_raw()
        ) != 0
    }
}
