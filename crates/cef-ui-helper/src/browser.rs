use crate::{Frame, ref_counted_ptr, try_c};
use anyhow::Result;
use cef_ui_sys::cef_browser_t;

ref_counted_ptr!(Browser, cef_browser_t);

impl Browser {
    pub fn get_main_frame(&self) -> Result<Option<Frame>> {
        try_c!(self, get_main_frame, {
            Ok(Frame::from_ptr(get_main_frame(self.as_ptr())))
        })
    }
}
