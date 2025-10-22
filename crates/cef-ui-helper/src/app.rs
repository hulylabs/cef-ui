use crate::{
    RefCountedPtr, RenderProcessHandler, SchemeRegistrar, Wrappable, Wrapped, ref_counted_ptr
};
use cef_ui_sys::{cef_app_t, cef_render_process_handler_t, cef_scheme_registrar_t};
use std::{mem::zeroed, ptr::null_mut};

pub trait AppCallbacks: Send + Sync + 'static {
    fn on_register_custom_schemes(&mut self, _registrar: SchemeRegistrar) {}

    fn get_render_process_handler(&mut self) -> Option<RenderProcessHandler> {
        None
    }
}

ref_counted_ptr!(App, cef_app_t);

impl App {
    pub fn new<C: AppCallbacks>(delegate: C) -> Self {
        Self(AppWrapper::new(delegate).wrap())
    }
}

struct AppWrapper(Box<dyn AppCallbacks>);

impl AppWrapper {
    pub fn new<C: AppCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }

    unsafe extern "C" fn c_on_register_custom_schemes(
        this: *mut cef_app_t,
        registrar: *mut cef_scheme_registrar_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let registrar = SchemeRegistrar::from_ptr_unchecked(registrar);
        this.0
            .on_register_custom_schemes(registrar);
    }

    unsafe extern "C" fn c_get_render_process_handler(
        this: *mut cef_app_t
    ) -> *mut cef_render_process_handler_t {
        let this: &mut Self = Wrapped::wrappable(this);

        this.0
            .get_render_process_handler()
            .map(|handler| handler.into_raw())
            .unwrap_or_else(null_mut)
    }
}

impl Wrappable for AppWrapper {
    type Cef = cef_app_t;

    fn wrap(self) -> RefCountedPtr<cef_app_t> {
        RefCountedPtr::wrap(
            cef_app_t {
                base: unsafe { zeroed() },

                on_before_command_line_processing: None,
                on_register_custom_schemes:        Some(Self::c_on_register_custom_schemes),
                get_resource_bundle_handler:       None,
                get_browser_process_handler:       None,
                get_render_process_handler:        Some(Self::c_get_render_process_handler)
            },
            self
        )
    }
}
