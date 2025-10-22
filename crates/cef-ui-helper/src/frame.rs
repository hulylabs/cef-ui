use crate::{CEFLIB, ProcessId, ProcessMessage, ref_counted_ptr};
use anyhow::Result;
use cef_ui_sys::cef_frame_t;

ref_counted_ptr!(Frame, cef_frame_t);

impl Frame {
    pub fn is_main(&self) -> Result<bool> {
        unsafe {
            let lib = &CEFLIB;
            Ok((lib.cef_frame_is_main)(self.as_ptr()) != 0)
        }
    }

    pub fn send_process_message(
        &self,
        target_process: ProcessId,
        message: ProcessMessage
    ) -> Result<bool> {
        unsafe {
            let lib = &CEFLIB;
            let result = (lib.cef_frame_send_process_message)(
                self.as_ptr(),
                target_process.into(),
                message.into_raw()
            );
            Ok(result != 0)
        }
    }
}
