use crate::{CefString, ref_counted_ptr, try_c};
use anyhow::Result;
use cef_ui_sys::cef_list_value_t;

ref_counted_ptr!(ListValue, cef_list_value_t);

impl ListValue {
    pub fn set_string(&self, index: usize, value: &str) -> Result<bool> {
        try_c!(self, set_string, {
            let value = CefString::new(value);

            Ok(set_string(self.as_ptr(), index, value.as_ptr()) != 0)
        })
    }
}
