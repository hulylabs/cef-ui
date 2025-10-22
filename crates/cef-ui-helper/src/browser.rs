use crate::{CEFLIB, Frame, ref_counted_ptr};
use anyhow::Result;
use cef_ui_sys::cef_browser_t;

ref_counted_ptr!(Browser, cef_browser_t);

impl Browser {
    pub fn get_main_frame(&self) -> Result<Option<Frame>> {
        unsafe {
            let lib = &CEFLIB;
            let frame = (lib.cef_browser_get_main_frame)(self.as_ptr());
            Ok(Frame::from_ptr(frame))
        }
    }
}
