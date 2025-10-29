use anyhow::Result;
use std::{mem::zeroed, ptr::null_mut};

use crate::{CEFLIB, CefString, RefCountedPtr, Wrappable, Wrapped, ref_counted_ptr, try_c};
use cef_ui_sys::{
    cef_string_t, cef_v8_propertyattribute_t, cef_v8context_t, cef_v8handler_t, cef_v8value_t
};

ref_counted_ptr!(V8Context, cef_v8context_t);

impl V8Context {
    pub fn get_global(&self) -> Result<V8Value> {
        try_c!(self, get_global, {
            Ok(V8Value::from_ptr_unchecked(get_global(self.as_ptr())))
        })
    }

    pub fn enter(&self) -> Result<i32> {
        try_c!(self, enter, { Ok(enter(self.as_ptr())) })
    }

    pub fn exit(&self) -> Result<i32> {
        try_c!(self, exit, { Ok(exit(self.as_ptr())) })
    }

    pub fn eval(
        &self,
        code: &str,
        script_url: &str,
        start_line: i32,
        retval: &mut V8Value
    ) -> Result<bool> {
        try_c!(self, eval, {
            let mut retval_raw = null_mut();
            let mut exception_raw = null_mut();

            let result = eval(
                self.as_ptr(),
                CefString::new(code).as_ptr(),
                CefString::new(script_url).as_ptr(),
                start_line,
                &mut retval_raw,
                &mut exception_raw
            );

            *retval = V8Value::from_ptr_unchecked(retval_raw);
            Ok(result != 0)
        })
    }
}

pub trait V8HandlerCallbacks: Send + Sync + 'static {
    fn execute(
        &mut self,
        name: String,
        object: V8Value,
        arguments_count: usize,
        arguments: Vec<V8Value>
    ) -> Result<i32>;
}

ref_counted_ptr!(V8Handler, cef_v8handler_t);

impl V8Handler {
    pub fn new<C: V8HandlerCallbacks>(delegate: C) -> Self {
        Self(V8HandlerWrapper::new(delegate).wrap())
    }
}

struct V8HandlerWrapper(Box<dyn V8HandlerCallbacks>);

impl V8HandlerWrapper {
    pub fn new<C: V8HandlerCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }

    unsafe extern "C" fn execute(
        this: *mut cef_v8handler_t,
        name: *const cef_string_t,
        object: *mut cef_v8value_t,
        arguments_count: usize,
        arguments: *const *mut cef_v8value_t,
        _retval: *mut *mut cef_v8value_t,
        _exception: *mut cef_string_t
    ) -> std::os::raw::c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let name = CefString::from_ptr_unchecked(name).into();
        let object: V8Value = V8Value::from_ptr_unchecked(object);
        let arguments = if arguments_count > 0 {
            std::slice::from_raw_parts(arguments, arguments_count)
                .iter()
                .map(|&arg| V8Value::from_ptr_unchecked(arg))
                .collect()
        } else {
            vec![]
        };

        this.0
            .execute(name, object, arguments_count, arguments)
            .unwrap()
    }
}

impl Wrappable for V8HandlerWrapper {
    type Cef = cef_v8handler_t;

    fn wrap(self) -> RefCountedPtr<cef_v8handler_t> {
        RefCountedPtr::wrap(
            cef_v8handler_t {
                base:    unsafe { zeroed() },
                execute: Some(Self::execute)
            },
            self
        )
    }
}

ref_counted_ptr!(V8Value, cef_v8value_t);

impl V8Value {
    pub fn get_value_by_key(&self, key: &str) -> Result<Self> {
        try_c!(self, get_value_bykey, {
            Ok(V8Value::from_ptr_unchecked(get_value_bykey(
                self.as_ptr(),
                CefString::new(key).as_ptr()
            )))
        })
    }

    pub fn get_string_value(&self) -> Result<String> {
        try_c!(self, get_string_value, {
            let s = get_string_value(self.as_ptr());
            let result = match CefString::from_userfree_ptr(s) {
                Some(str) => str.into(),
                None => Err(anyhow::anyhow!("string is empty"))?
            };
            Ok(result)
        })
    }

    pub fn set_value_by_key(&self, key: &str, value: Self) -> Result<bool> {
        try_c!(self, set_value_bykey, {
            Ok(set_value_bykey(
                self.as_ptr(),
                CefString::new(key).as_ptr(),
                value.into_raw(),
                cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_NONE
            ) != 0)
        })
    }

    pub fn create_string(s: &str) -> Self {
        unsafe {
            let lib = &CEFLIB;
            let val = (lib.cef_v8_value_create_string)(CefString::new(s).as_ptr());
            V8Value::from_ptr_unchecked(val)
        }
    }

    pub fn create_function(name: &str, handler: V8Handler) -> Result<V8Value> {
        unsafe {
            let lib = &CEFLIB;
            let val =
                (lib.cef_v8_value_create_function)(CefString::new(name).as_ptr(), handler.as_ptr());
            Ok(V8Value::from_ptr_unchecked(val))
        }
    }
}
