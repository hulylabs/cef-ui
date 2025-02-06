
use std::mem::zeroed;

use anyhow::Result;

use cef_ui_sys::{cef_string_t, cef_v8_propertyattribute_t, cef_v8context_get_current_context, cef_v8context_get_entered_context, cef_v8context_in_context, cef_v8context_t, cef_v8exception_t, cef_v8handler_t, cef_v8value_create_function, cef_v8value_create_int, cef_v8value_create_string, cef_v8value_t, _SC_TYPED_MEMORY_OBJECTS};

use crate::{ref_counted_ptr, try_c, Browser, CefString, Frame, RefCountedPtr, Wrappable, Wrapped};

ref_counted_ptr!(V8Context, cef_v8context_t);

/// Structure representing a V8 context handle. V8 handles can only be accessed
/// from the thread on which they are created. Valid threads for creating a V8
/// handle include the render process main thread (TID_RENDERER) and WebWorker
/// threads. A task runner for posting tasks on the associated thread can be
/// retrieved via the cef_v8context_t::get_task_runner() function.
impl V8Context {
    // TODO: Create a TaskRunner wrapper for _cef_task_runner_t
    // /// Returns the task runner associated with this context. V8 handles can only
    // /// be accessed from the thread on which they are created. This function can
    // /// be called on any render process thread.
    // pub fn get_task_runner(&self) -> Result<TaskRunner> {
    //     try_c!(self, get_task_runner, { Ok(TaskRunner::from_ptr_unchecked(get_task_runner(self.as_ptr()))) })
    // }

    /// Returns true (1) if the underlying handle is valid and it can be accessed
    /// on the current thread. Do not call any other functions if this function
    /// returns false (0).
    pub fn is_valid(&self) -> Result<bool> {
        try_c!(self, is_valid, { Ok(is_valid(self.as_ptr()) != 0) })
    }

    /// Returns the browser for this context. This function will return an NULL
    /// reference for WebWorker contexts.
    pub fn get_browser(&self) -> Result<Browser> {
        try_c!(self, get_browser, { Ok(Browser::from_ptr_unchecked(get_browser(self.as_ptr()))) })
    }

    /// Returns the frame for this context. This function will return an NULL
    /// reference for WebWorker contexts.
    pub fn get_frame(&self) -> Result<Frame> {
        try_c!(self, get_frame, { Ok(Frame::from_ptr_unchecked(get_frame(self.as_ptr()))) })
    }

    /// Returns the global object for this context. The context must be entered
    /// before calling this function.
    pub fn get_global(&self) -> Result<V8Value> {
        try_c!(self, get_global, { Ok(V8Value::from_ptr_unchecked(get_global(self.as_ptr()))) })
    }

    /// Enter this context. A context must be explicitly entered before creating a
    /// V8 Object, Array, Function or Date asynchronously. exit() must be called
    /// the same number of times as enter() before releasing this context. V8
    /// objects belong to the context in which they are created. Returns true (1)
    /// if the scope was entered successfully.
    pub fn enter(&self) -> Result<i32> {
        try_c!(self, enter, { Ok(enter(self.as_ptr())) })
    }

    /// Exit this context. Call this function only after calling enter(). Returns
    /// true (1) if the scope was exited successfully.
    pub fn exit(&self) -> Result<i32> {
        try_c!(self, exit, { Ok(exit(self.as_ptr())) })
    }

    /// Returns true (1) if this object is pointing to the same handle as |that|
    /// object.
    pub fn is_same(&self, that: Self) -> Result<bool> {
        try_c!(self, is_same, { Ok(is_same(self.as_ptr(), that.into_raw()) != 0) })
    }

    // /// Execute a string of JavaScript code in this V8 context. The |script_url|
    // /// parameter is the URL where the script in question can be found, if any.
    // /// The |start_line| parameter is the base line number to use for error
    // /// reporting. On success |retval| will be set to the return value, if any,
    // /// and the function will return true (1). On failure |exception| will be set
    // /// to the exception, if any, and the function will return false (0).
    // pub fn eval(&self, code: &str, script_url: &str, start_line: i32, retval: &mut V8Value) -> Result<i32> {
    //     try_c!(self, eval, { 
    //         let mut retval_raw: cef_v8value_t = zeroed();
    //         let mut retval_raw: *mut cef_v8value_t = &mut retval_raw;
    //         let retval_raw: *mut *mut cef_v8value_t = &mut retval_raw;

    //         let result = eval(
    //             self.as_ptr(),
    //             CefString::new(code).as_ptr(),
    //             CefString::new(script_url).as_ptr(),
    //             start_line,
    //             retval_raw,
    //             null_mut());

    //         *retval = V8Value::from_ptr_unchecked(*retval_raw);
    //         Ok(result)
    //     })
    // }

    /// Returns the current (top) context object in the V8 context stack.
    pub fn get_current_context() -> Result<Self> {
        unsafe { Ok(V8Context::from_ptr_unchecked(cef_v8context_get_current_context())) }
    }

    /// Returns the entered (bottom) context object in the V8 context stack.
    pub fn get_entered_context() -> Result<Self> {
        unsafe { Ok(V8Context::from_ptr_unchecked(cef_v8context_get_entered_context())) }
    }

    /// Returns true (1) if V8 is currently inside a context.
    pub fn in_context() -> Result<bool> {
        unsafe { Ok(cef_v8context_in_context() != 0) }
    }
}

/// Interface that should be implemented to handle V8 function calls. The
/// methods of this class will be called on the thread associated with the V8
/// function.
pub trait V8HandlerCallbacks: Send + Sync + 'static {
    // TODO: Add arguments, retval and exception parameters
    /// Handle execution of the function identified by |name|. |object| is the
    /// receiver ('this' object) of the function. |arguments| is the list of
    /// arguments passed to the function. If execution succeeds set |retval| to
    /// the function return value. If execution fails set |exception| to the
    /// exception that will be thrown. Return true (1) if execution was handled.
    fn execute(&mut self, name: String, object: V8Value, arguments_count: usize) -> Result<i32>;
}

// Structure that should be implemented to handle V8 function calls. The
// functions of this structure will be called on the thread associated with the
// V8 function.
ref_counted_ptr!(V8Handler, cef_v8handler_t);

impl V8Handler {
    pub fn new<C: V8HandlerCallbacks>(delegate: C) -> Self {
        Self(V8HandlerWrapper::new(delegate).wrap())
    }
}

/// Translates CEF -> Rust callbacks.
struct V8HandlerWrapper(Box<dyn V8HandlerCallbacks>);

impl V8HandlerWrapper {
    pub fn new<C: V8HandlerCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }

    // TODO: use arguments, retval and exception
    /// Handle execution of the function identified by |name|. |object| is the
    /// receiver ('this' object) of the function. |arguments| is the list of
    /// arguments passed to the function. If execution succeeds set |retval| to
    /// the function return value. If execution fails set |exception| to the
    /// exception that will be thrown. Return true (1) if execution was handled.
    unsafe extern "C" fn execute(
        this: *mut cef_v8handler_t,
        name: *const cef_string_t,
        object: *mut cef_v8value_t,
        arguments_count: usize,
        arguments: *const *mut cef_v8value_t,
        retval: *mut *mut cef_v8value_t,
        exception: *mut cef_string_t
    ) -> std::os::raw::c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let name = CefString::from_ptr_unchecked(name).into();
        let object: V8Value = V8Value::from_ptr_unchecked(object);

        this.0
            .execute(name, object, arguments_count).unwrap()
    }
}

impl Wrappable for V8HandlerWrapper {
    type Cef = cef_v8handler_t;

    /// Converts this to a smart pointer.
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

// Structure representing a V8 value handle. V8 handles can only be accessed
// from the thread on which they are created. Valid threads for creating a V8
// handle include the render process main thread (TID_RENDERER) and WebWorker
// threads. A task runner for posting tasks on the associated thread can be
// retrieved via the cef_v8context_t::get_task_runner() function.
ref_counted_ptr!(V8Value, cef_v8value_t);

impl V8Value {
    /// Returns true (1) if the underlying handle is valid and it can be accessed
    /// on the current thread. Do not call any other functions if this function
    /// returns false (0).
    pub fn is_valid(&self) -> Result<bool> {
        try_c!(self, is_valid, { Ok(is_valid(self.as_ptr()) != 0) })
    }

    /// True if the value type is undefined.
    pub fn is_undefined(&self) -> Result<bool> {
        try_c!(self, is_undefined, { Ok(is_undefined(self.as_ptr()) != 0) })
    }

    /// True if the value type is null.
    pub fn is_null(&self) -> Result<bool> {
        try_c!(self, is_null, { Ok(is_null(self.as_ptr()) != 0) })
    }

    /// True if the value type is bool.
    pub fn is_bool(&self) -> Result<bool> {
        try_c!(self, is_bool, { Ok(is_bool(self.as_ptr()) != 0) })
    }

    /// True if the value type is int.
    pub fn is_int(&self) -> Result<bool> {
        try_c!(self, is_int, { Ok(is_int(self.as_ptr()) != 0) })
    }

    /// True if the value type is unsigned int.
    pub fn is_uint(&self) -> Result<bool> {
        try_c!(self, is_uint, { Ok(is_uint(self.as_ptr()) != 0) })
    }

    /// True if the value type is double.
    pub fn is_double(&self) -> Result<bool> {
        try_c!(self, is_double, { Ok(is_double(self.as_ptr()) != 0) })
    }

    /// True if the value type is Date.
    pub fn is_date(&self) -> Result<bool> {
        try_c!(self, is_date, { Ok(is_date(self.as_ptr()) != 0) })
    }

    /// True if the value type is string.
    pub fn is_string(&self) -> Result<bool> {
        try_c!(self, is_string, { Ok(is_string(self.as_ptr()) != 0) })
    }

    /// True if the value type is object.
    pub fn is_object(&self) -> Result<bool> {
        try_c!(self, is_object, { Ok(is_object(self.as_ptr()) != 0) })
    }

    /// True if the value type is array.
    pub fn is_array(&self) -> Result<bool> {
        try_c!(self, is_array, { Ok(is_array(self.as_ptr()) != 0) })
    }

    /// True if the value type is an ArrayBuffer.
    pub fn is_array_buffer(&self) -> Result<bool> {
        try_c!(self, is_array_buffer, { Ok(is_array_buffer(self.as_ptr()) != 0) })
    }

    /// True if the value type is function.
    pub fn is_function(&self) -> Result<bool> {
        try_c!(self, is_function, { Ok(is_function(self.as_ptr()) != 0) })
    }

    /// True if the value type is a Promise.
    pub fn is_promise(&self) -> Result<bool> {
        try_c!(self, is_promise, { Ok(is_promise(self.as_ptr()) != 0) })
    }

    /// Returns true (1) if this object is pointing to the same handle as |that|
    /// object.
    pub fn is_same(&self, that: &Self) -> Result<bool> {
        try_c!(self, is_same, { Ok(is_same(self.as_ptr(), that.as_ptr()) != 0) })
    }

    /// Return a bool value.
    pub fn get_bool_value(&self) -> Result<bool> {
        try_c!(self, get_bool_value, { Ok(get_bool_value(self.as_ptr()) != 0) })
    }

    /// Return an int value.
    pub fn get_int_value(&self) -> Result<i32> {
        try_c!(self, get_int_value, { Ok(get_int_value(self.as_ptr())) })
    }

    /// Return an unsigned int value.
    pub fn get_uint_value(&self) -> Result<u32> {
        try_c!(self, get_uint_value, { Ok(get_uint_value(self.as_ptr())) })
    }

    /// Return a double value.
    pub fn get_double_value(&self) -> Result<f64> {
        try_c!(self, get_double_value, { Ok(get_double_value(self.as_ptr())) })
    }

    /// Return a Date value.
    // pub fn get_date_value(&self) -> Result<f64> {
    //     try_c!(self, get_date_value, { Ok(get_date_value(self.as_ptr())) })
    // }

    /// Return a string value.
    pub fn get_string_value(&self) -> Result<String> {
        try_c!(self, get_string_value, { Ok(CefString::from_userfree_ptr_unchecked(get_string_value(self.as_ptr())).into()) })
    }

    /// Returns true (1) if this is a user created object.
    pub fn is_user_created(&self) -> Result<bool> {
        try_c!(self, is_user_created, { Ok(is_user_created(self.as_ptr()) != 0) })
    }

    /// Returns true (1) if the last function call resulted in an exception. This
    /// attribute exists only in the scope of the current CEF value object.
    pub fn has_exception(&self) -> Result<bool> {
        try_c!(self, has_exception, { Ok(has_exception(self.as_ptr()) != 0) })
    }

    /// Returns the exception resulting from the last function call. This
    /// attribute exists only in the scope of the current CEF value object.
    // pub fn get_exception(&self) -> Result<CefString> {
    //     try_c!(self, get_exception, { Ok(V8Exception::from_ptr_unchecked(get_exception(self.as_ptr()))) })
    // }

    /// Clears the last exception and returns true (1) on success.
    pub fn clear_exception(&self) -> Result<bool> {
        try_c!(self, clear_exception, { Ok(clear_exception(self.as_ptr()) != 0) })
    }

    /// Returns true (1) if this object will re-throw future exceptions. This
    /// attribute exists only in the scope of the current CEF value object.
    pub fn will_rethrow_exceptions(&self) -> Result<bool> {
        try_c!(self, will_rethrow_exceptions, { Ok(will_rethrow_exceptions(self.as_ptr()) != 0) })
    }

    /// Set whether this object will re-throw future exceptions. By default
    /// exceptions are not re-thrown. If a exception is re-thrown the current
    /// context should not be accessed again until after the exception has been
    /// caught and not re-thrown. Returns true (1) on success. This attribute
    /// exists only in the scope of the current CEF value object.
    pub fn set_rethrow_exceptions(&self, rethrow: bool) -> Result<bool> {
        try_c!(self, set_rethrow_exceptions, { Ok(set_rethrow_exceptions(self.as_ptr(), rethrow as i32) != 0) })
    }

    /// Returns true (1) if the object has a value with the specified identifier.
    pub fn has_value_by_key(&self, key: &str) -> Result<bool> {
        try_c!(self, has_value_bykey, { Ok(has_value_bykey(self.as_ptr(), CefString::new(key).as_ptr()) != 0) })
    }

    /// Returns true (1) if the object has a value with the specified identifier.
    pub fn has_value_by_index(&self, index: i32) -> Result<bool> {
        try_c!(self, has_value_byindex, { Ok(has_value_byindex(self.as_ptr(), index) != 0) })
    }

    /// Deletes the value with the specified identifier and returns true (1) on
    /// success. Returns false (0) if this function is called incorrectly or an
    /// exception is thrown. For read-only and don't-delete values this function
    /// will return true (1) even though deletion failed.
    pub fn delete_value_by_key(&self, key: &str) -> Result<bool> {
        try_c!(self, delete_value_bykey, { Ok(delete_value_bykey(self.as_ptr(), CefString::new(key).as_ptr()) != 0) })
    }

    /// Deletes the value with the specified identifier and returns true (1) on
    /// success. Returns false (0) if this function is called incorrectly,
    /// deletion fails or an exception is thrown. For read-only and don't-delete
    /// values this function will return true (1) even though deletion failed.
    pub fn delete_value_by_index(&self, index: i32) -> Result<bool> {
        try_c!(self, delete_value_byindex, { Ok(delete_value_byindex(self.as_ptr(), index) != 0) })
    }

    /// Returns the value with the specified identifier on success. Returns NULL
    /// if this function is called incorrectly or an exception is thrown.
    pub fn get_value_by_key(&self, key: &str) -> Result<Self> {
        try_c!(self, get_value_bykey, { Ok(V8Value::from_ptr_unchecked(get_value_bykey(self.as_ptr(), CefString::new(key).as_ptr()))) })
    }

    /// Returns the value with the specified identifier on success. Returns NULL
    /// if this function is called incorrectly or an exception is thrown.
    pub fn get_value_by_index(&self, index: i32) -> Result<Self> {
        try_c!(self, get_value_byindex, { Ok(V8Value::from_ptr_unchecked(get_value_byindex(self.as_ptr(), index))) })
    }

    // TODO: create a wrapper for cef_v8_propertyattribute_t and pass it as parameter
    /// Associates a value with the specified identifier and returns true (1) on
    /// success. Returns false (0) if this function is called incorrectly or an
    /// exception is thrown. For read-only values this function will return true
    /// (1) even though assignment failed.
    pub fn set_value_by_key(&self, key: &str, value: Self) -> Result<bool> {
        try_c!(self, set_value_bykey, { Ok(set_value_bykey(self.as_ptr(), CefString::new(key).as_ptr(), value.into_raw(), cef_v8_propertyattribute_t::V8_PROPERTY_ATTRIBUTE_NONE) != 0) })
    }

    /// Associates a value with the specified identifier and returns true (1) on
    /// success. Returns false (0) if this function is called incorrectly or an
    /// exception is thrown. For read-only values this function will return true
    /// (1) even though assignment failed.
    pub fn set_value_by_index(&self, index: i32, value: Self) -> Result<bool> {
        try_c!(self, set_value_byindex, { Ok(set_value_byindex(self.as_ptr(), index, value.into_raw()) != 0) })
    }

    // /// Registers an identifier and returns true (1) on success. Access to the
    // /// identifier will be forwarded to the cef_v8accessor_t instance passed to
    // /// cef_v8value_t::cef_v8value_create_object(). Returns false (0) if this
    // /// function is called incorrectly or an exception is thrown. For read-only
    // /// values this function will return true (1) even though assignment failed.
    // int(CEF_CALLBACK* set_value_byaccessor)(struct _cef_v8value_t* self,
    //     const cef_string_t* key,
    //     cef_v8_accesscontrol_t settings,
    //     cef_v8_propertyattribute_t attribute);

    // ///
    // /// Read the keys for the object's values into the specified vector. Integer-
    // /// based keys will also be returned as strings.
    // int(CEF_CALLBACK* get_keys)(struct _cef_v8value_t* self,
    //     cef_string_list_t keys);

    /// Sets the user data for this object and returns true (1) on success.
    /// Returns false (0) if this function is called incorrectly. This function
    /// can only be called on user created objects.
    // int(CEF_CALLBACK* set_user_data)(struct _cef_v8value_t* self,
    //     struct _cef_base_ref_counted_t* user_data);

    // /// Returns the user data, if any, assigned to this object.
    // struct _cef_base_ref_counted_t*(CEF_CALLBACK* get_user_data)(
    //     struct _cef_v8value_t* self);

    // /// Returns the amount of externally allocated memory registered for the
    // /// object.
    // int(CEF_CALLBACK* get_externally_allocated_memory)(
    //     struct _cef_v8value_t* self);

    // /// Adjusts the amount of registered external memory for the object. Used to
    // /// give V8 an indication of the amount of externally allocated memory that is
    // /// kept alive by JavaScript objects. V8 uses this information to decide when
    // /// to perform global garbage collection. Each cef_v8value_t tracks the amount
    // /// of external memory associated with it and automatically decreases the
    // /// global total by the appropriate amount on its destruction.
    // /// |change_in_bytes| specifies the number of bytes to adjust by. This
    // /// function returns the number of bytes associated with the object after the
    // /// adjustment. This function can only be called on user created objects.
    // int(CEF_CALLBACK* adjust_externally_allocated_memory)(
    //     struct _cef_v8value_t* self,
    //     int change_in_bytes);

    // /// Returns the number of elements in the array.
    // int(CEF_CALLBACK* get_array_length)(struct _cef_v8value_t* self);

    // /// Returns the ReleaseCallback object associated with the ArrayBuffer or NULL
    // /// if the ArrayBuffer was not created with CreateArrayBuffer.
    // struct _cef_v8array_buffer_release_callback_t*(
    //     CEF_CALLBACK* get_array_buffer_release_callback)(
    //     struct _cef_v8value_t* self);

    // /// Prevent the ArrayBuffer from using it's memory block by setting the length
    // /// to zero. This operation cannot be undone. If the ArrayBuffer was created
    // /// with CreateArrayBuffer then
    // /// cef_v8array_buffer_release_callback_t::ReleaseBuffer will be called to
    // /// release the underlying buffer.
    // int(CEF_CALLBACK* neuter_array_buffer)(struct _cef_v8value_t* self);

    // /// Returns the length (in bytes) of the ArrayBuffer.
    // size_t(CEF_CALLBACK* get_array_buffer_byte_length)(
    //     struct _cef_v8value_t* self);

    // /// Returns a pointer to the beginning of the memory block for this
    // /// ArrayBuffer backing store. The returned pointer is valid as long as the
    // /// cef_v8value_t is alive.
    // void*(CEF_CALLBACK* get_array_buffer_data)(struct _cef_v8value_t* self);

    // /// Returns the function name.
    // cef_string_userfree_t(CEF_CALLBACK* get_function_name)(
    //     struct _cef_v8value_t* self);

    // /// Returns the function handler or NULL if not a CEF-created function.
    // struct _cef_v8handler_t*(CEF_CALLBACK* get_function_handler)(
    //     struct _cef_v8value_t* self);

    // /// Execute the function using the current V8 context. This function should
    // /// only be called from within the scope of a cef_v8handler_t or
    // /// cef_v8accessor_t callback, or in combination with calling enter() and
    // /// exit() on a stored cef_v8context_t reference. |object| is the receiver
    // /// ('this' object) of the function. If |object| is NULL the current context's
    // /// global object will be used. |arguments| is the list of arguments that will
    // /// be passed to the function. Returns the function return value on success.
    // /// Returns NULL if this function is called incorrectly or an exception is
    // /// thrown.
    // struct _cef_v8value_t*(CEF_CALLBACK* execute_function)(
    //     struct _cef_v8value_t* self,
    //     struct _cef_v8value_t* object,
    //     size_t argumentsCount,
    //     struct _cef_v8value_t* const* arguments);

    // /// Execute the function using the specified V8 context. |object| is the
    // /// receiver ('this' object) of the function. If |object| is NULL the
    // /// specified context's global object will be used. |arguments| is the list of
    // /// arguments that will be passed to the function. Returns the function return
    // /// value on success. Returns NULL if this function is called incorrectly or
    // /// an exception is thrown.
    // struct _cef_v8value_t*(CEF_CALLBACK* execute_function_with_context)(
    //     struct _cef_v8value_t* self,
    //     struct _cef_v8context_t* context,
    //     struct _cef_v8value_t* object,
    //     size_t argumentsCount,
    //     struct _cef_v8value_t* const* arguments);

    // /// Resolve the Promise using the current V8 context. This function should
    // /// only be called from within the scope of a cef_v8handler_t or
    // /// cef_v8accessor_t callback, or in combination with calling enter() and
    // /// exit() on a stored cef_v8context_t reference. |arg| is the argument passed
    // /// to the resolved promise. Returns true (1) on success. Returns false (0) if
    // /// this function is called incorrectly or an exception is thrown.
    // int(CEF_CALLBACK* resolve_promise)(struct _cef_v8value_t* self,
    //     struct _cef_v8value_t* arg);

    // /// Reject the Promise using the current V8 context. This function should only
    // /// be called from within the scope of a cef_v8handler_t or cef_v8accessor_t
    // /// callback, or in combination with calling enter() and exit() on a stored
    // /// cef_v8context_t reference. Returns true (1) on success. Returns false (0)
    // /// if this function is called incorrectly or an exception is thrown.
    // int(CEF_CALLBACK* reject_promise)(struct _cef_v8value_t* self,
    //     const cef_string_t* errorMsg);


    // /// Create a new cef_v8value_t object of type undefined.
    // CEF_EXPORT cef_v8value_t* cef_v8value_create_undefined(void);

    // /// Create a new cef_v8value_t object of type null.
    // CEF_EXPORT cef_v8value_t* cef_v8value_create_null(void);

    // /// Create a new cef_v8value_t object of type bool.
    // CEF_EXPORT cef_v8value_t* cef_v8value_create_bool(int value);
  
    /// Create a new cef_v8value_t object of type int.
    pub fn create_int(i: i32) -> Self {
        unsafe { V8Value::from_ptr_unchecked(cef_v8value_create_int(i)) }
    }

    // /// Create a new cef_v8value_t object of type unsigned int.
    // CEF_EXPORT cef_v8value_t* cef_v8value_create_uint(uint32_t value);

    // /// Create a new cef_v8value_t object of type double.
    // CEF_EXPORT cef_v8value_t* cef_v8value_create_double(double value);

    // /// Create a new cef_v8value_t object of type Date. This function should only be
    // /// called from within the scope of a cef_render_process_handler_t,
    // /// cef_v8handler_t or cef_v8accessor_t callback, or in combination with calling
    // /// enter() and exit() on a stored cef_v8context_t reference.
    // CEF_EXPORT cef_v8value_t* cef_v8value_create_date(cef_basetime_t date);

    pub fn create_string(s: CefString) -> Self {
        unsafe { V8Value::from_ptr_unchecked(cef_v8value_create_string(s.as_ptr())) }
    }

    // /// Create a new cef_v8value_t object of type object with optional accessor
    // /// and/or interceptor. This function should only be called from within the
    // /// scope of a cef_render_process_handler_t, cef_v8handler_t or cef_v8accessor_t
    // /// callback, or in combination with calling enter() and exit() on a stored
    // /// cef_v8context_t reference.
    // CEF_EXPORT cef_v8value_t* cef_v8value_create_object(
    //     cef_v8accessor_t* accessor,
    //     cef_v8interceptor_t* interceptor);

    // /// Create a new cef_v8value_t object of type array with the specified |length|.
    // /// If |length| is negative the returned array will have length 0. This function
    // /// should only be called from within the scope of a
    // /// cef_render_process_handler_t, cef_v8handler_t or cef_v8accessor_t callback,
    // /// or in combination with calling enter() and exit() on a stored
    // /// cef_v8context_t reference.
    // CEF_EXPORT cef_v8value_t* cef_v8value_create_array(int length);

    // /// Create a new cef_v8value_t object of type ArrayBuffer which wraps the
    // /// provided |buffer| of size |length| bytes. The ArrayBuffer is externalized,
    // /// meaning that it does not own |buffer|. The caller is responsible for freeing
    // /// |buffer| when requested via a call to
    // /// cef_v8array_buffer_release_callback_t::ReleaseBuffer. This function should
    // /// only be called from within the scope of a cef_render_process_handler_t,
    // /// cef_v8handler_t or cef_v8accessor_t callback, or in combination with calling
    // /// enter() and exit() on a stored cef_v8context_t reference.
    // CEF_EXPORT cef_v8value_t* cef_v8value_create_array_buffer(
    //     void* buffer,
    //     size_t length,
    //     cef_v8array_buffer_release_callback_t* release_callback);

    /// Create a new cef_v8value_t object of type function. This function should
    /// only be called from within the scope of a cef_render_process_handler_t,
    /// cef_v8handler_t or cef_v8accessor_t callback, or in combination with calling
    /// enter() and exit() on a stored cef_v8context_t reference.
    pub fn create_function(name: &str, handler: V8Handler) -> Result<V8Value> {
       unsafe { Ok(V8Value::from_ptr_unchecked(cef_v8value_create_function(CefString::new(name).as_ptr(), handler.as_ptr()))) }
    }

    // /// Create a new cef_v8value_t object of type Promise. This function should only
    // /// be called from within the scope of a cef_render_process_handler_t,
    // /// cef_v8handler_t or cef_v8accessor_t callback, or in combination with calling
    // /// enter() and exit() on a stored cef_v8context_t reference.
    // CEF_EXPORT cef_v8value_t* cef_v8value_create_promise(void);

}
