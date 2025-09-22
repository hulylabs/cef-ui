use crate::{RefCountedPtr, SchemeRegistrar, Wrappable, Wrapped, ref_counted_ptr};
use cef_ui_sys::{cef_app_t, cef_scheme_registrar_t};
use std::mem::zeroed;

/// Implement this structure to provide handler implementations. Methods will be
/// called by the process and/or thread indicated.
pub trait AppCallbacks: Send + Sync + 'static {
    /// Provides an opportunity to register custom schemes. Do not keep a
    /// reference to the |registrar| object. This function is called on the main
    /// thread for each process and the registered schemes should be the same
    /// across all processes.
    fn on_register_custom_schemes(&mut self, _registrar: SchemeRegistrar) {}
}

// Implement this structure to provide handler implementations. Methods will be
// called by the process and/or thread indicated.
ref_counted_ptr!(App, cef_app_t);

impl App {
    pub fn new<C: AppCallbacks>(delegate: C) -> Self {
        Self(AppWrapper::new(delegate).wrap())
    }
}

/// Translates CEF -> Rust callbacks.
struct AppWrapper(Box<dyn AppCallbacks>);

#[allow(dead_code)]
#[allow(unused_variables)]
impl AppWrapper {
    pub fn new<C: AppCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }

    /// Provides an opportunity to register custom schemes. Do not keep a
    /// reference to the |registrar| object. This function is called on the main
    /// thread for each process and the registered schemes should be the same
    /// across all processes.
    unsafe extern "C" fn c_on_register_custom_schemes(
        this: *mut cef_app_t,
        registrar: *mut cef_scheme_registrar_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let registrar = SchemeRegistrar::from_ptr_unchecked(registrar);
        this.0
            .on_register_custom_schemes(registrar);
    }
}

impl Wrappable for AppWrapper {
    type Cef = cef_app_t;

    /// Converts this to a smart pointer.
    fn wrap(self) -> RefCountedPtr<cef_app_t> {
        RefCountedPtr::wrap(
            cef_app_t {
                base: unsafe { zeroed() },

                on_before_command_line_processing: None,
                on_register_custom_schemes:        Some(Self::c_on_register_custom_schemes),
                get_resource_bundle_handler:       None,
                get_browser_process_handler:       None,
                get_render_process_handler:        None
            },
            self
        )
    }
}
