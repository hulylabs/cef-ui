use std::mem::zeroed;

use cef_ui_sys::{cef_post_task, cef_task_t};

use crate::{RefCountedPtr, ThreadId, Wrappable, Wrapped, ref_counted_ptr};

pub trait CefTaskCallbacks: Send + Sync + 'static {
    fn execute(&mut self);
}

ref_counted_ptr!(CefTask, cef_task_t);

impl CefTask {
    pub fn new<C: CefTaskCallbacks>(delegate: C) -> Self {
        Self(CefTaskWrapper::new(delegate).wrap())
    }
}

/// Translates CEF -> Rust callbacks.
struct CefTaskWrapper(Box<dyn CefTaskCallbacks>);

impl CefTaskWrapper {
    pub fn new<C: CefTaskCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }

    unsafe extern "C" fn execute(this: *mut cef_task_t) {
        let this: &mut Self = Wrapped::wrappable(this);
        this.0.execute();
    }
}

impl Wrappable for CefTaskWrapper {
    type Cef = cef_task_t;

    /// Converts this to a smart pointer.
    fn wrap(self) -> RefCountedPtr<cef_task_t> {
        RefCountedPtr::wrap(
            cef_task_t {
                base:    unsafe { zeroed() },
                execute: Some(Self::execute)
            },
            self
        )
    }
}

pub fn post_task(tid: ThreadId, task: CefTask) -> bool {
    unsafe { cef_post_task(tid.into(), task.into_raw()) != 0 }
}
