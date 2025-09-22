use cef_ui_sys::cef_scheme_registrar_t;

use crate::CefString;

pub struct SchemeRegistrar(pub *mut cef_scheme_registrar_t);

/// Class that manages custom scheme registrations.
impl SchemeRegistrar {
    pub fn from_ptr_unchecked(ptr: *mut cef_scheme_registrar_t) -> Self {
        Self(ptr)
    }

    /// Register a custom scheme. This method should not be called for the
    /// built-in HTTP, HTTPS, FILE, FTP, ABOUT and DATA schemes.
    ///
    /// See cef_scheme_options_t for possible values for |options|.
    ///
    /// This function may be called on any thread. It should only be called once
    /// per unique |scheme_name| value. If |scheme_name| is already registered or
    /// if an error occurs this method will return false.
    pub fn add_custom_scheme(&self, scheme_name: &str, options: i32) -> bool {
        unsafe {
            self.0
                .as_mut()
                .and_then(|this| {
                    this.add_custom_scheme
                        .map(|add_custom_scheme| {
                            let name = CefString::new(scheme_name);
                            add_custom_scheme(this, name.as_ptr(), options) != 0
                        })
                })
                .unwrap_or(false)
        }
    }
}

pub enum SchemeOptions {
    None = 0,
    Standard = 1,
    Local = 2,
    DisplayIsolated = 4,
    Secure = 8,
    CorsEnabled = 16,
    CspBypassing = 32,
    FetchEnabled = 64
}

impl Into<i32> for SchemeOptions {
    fn into(self) -> i32 {
        self as i32
    }
}
