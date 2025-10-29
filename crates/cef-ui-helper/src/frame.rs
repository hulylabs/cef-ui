use crate::{ProcessId, ProcessMessage, V8Context, ref_counted_ptr, try_c};
use anyhow::Result;
use cef_ui_sys::cef_frame_t;

ref_counted_ptr!(Frame, cef_frame_t);

impl Frame {
    pub fn is_main(&self) -> Result<bool> {
        try_c!(self, is_main, { Ok(is_main(self.as_ptr()) != 0) })
    }

    pub fn get_v8context(&self) -> Result<V8Context> {
        try_c!(self, get_v8context, {
            Ok(V8Context::from_ptr_unchecked(get_v8context(self.as_ptr())))
        })
    }

    pub fn send_process_message(
        &self,
        target_process: ProcessId,
        message: ProcessMessage
    ) -> Result<()> {
        try_c!(self, send_process_message, {
            Ok(send_process_message(
                self.as_ptr(),
                target_process.into(),
                message.into_raw()
            ))
        })
    }
}
