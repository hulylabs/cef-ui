use crate::{CEFLIB, CefString, ListValue, ref_counted_ptr, try_c};
use anyhow::Result;
use cef_ui_sys::{cef_process_id_t, cef_process_message_t};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ProcessId {
    Browser,
    Renderer
}

impl From<cef_process_id_t> for ProcessId {
    fn from(value: cef_process_id_t) -> Self {
        Self::from(&value)
    }
}

impl From<&cef_process_id_t> for ProcessId {
    fn from(value: &cef_process_id_t) -> Self {
        match value {
            cef_process_id_t::PID_BROWSER => ProcessId::Browser,
            cef_process_id_t::PID_RENDERER => ProcessId::Renderer
        }
    }
}

impl From<ProcessId> for cef_process_id_t {
    fn from(value: ProcessId) -> Self {
        Self::from(&value)
    }
}

impl From<&ProcessId> for cef_process_id_t {
    fn from(value: &ProcessId) -> Self {
        match value {
            ProcessId::Browser => cef_process_id_t::PID_BROWSER,
            ProcessId::Renderer => cef_process_id_t::PID_RENDERER
        }
    }
}

ref_counted_ptr!(ProcessMessage, cef_process_message_t);

impl ProcessMessage {
    pub fn new(name: &str) -> Self {
        unsafe {
            let lib = &CEFLIB;
            let msg = (lib.cef_process_message_create)(CefString::new(name).as_ptr());

            Self::from_ptr_unchecked(msg)
        }
    }

    pub fn get_argument_list(&self) -> Result<Option<ListValue>> {
        try_c!(self, get_argument_list, {
            Ok(ListValue::from_ptr(get_argument_list(self.as_ptr())))
        })
    }
}
