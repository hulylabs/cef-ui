use cef_ui_sys::cef_registration_t;

use crate::ref_counted_ptr;

// Generic callback interface used for managing the lifespan of a registration.
ref_counted_ptr!(Registration, cef_registration_t);
