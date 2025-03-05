use std::mem::zeroed;

use cef_ui_sys::{cef_post_task, cef_task_t};

use crate::{RefCountedPtr, ThreadId, Wrappable, Wrapped, ref_counted_ptr};

/// Implement this structure for asynchronous task execution. If the task is
/// posted successfully and if the associated message loop is still running then
/// the execute() function will be called on the target thread. If the task
/// fails to post then the task object may be destroyed on the source thread
/// instead of the target thread. For this reason be cautious when performing
/// work in the task object destructor.
pub trait CefTaskCallbacks: Send + Sync + 'static {
    /// Method that will be executed on the target thread.
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

    /// Method that will be executed on the target thread.
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

/// Post a task for execution on the specified thread. Equivalent to using
/// cef_task_runner_t::GetForThread(threadId)->PostTask(task).
pub fn post_task(tid: ThreadId, task: CefTask) -> bool {
    unsafe { cef_post_task(tid.into(), task.into_raw()) != 0 }
}
