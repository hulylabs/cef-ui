use crate::{
    Callback, CefString, RefCountedPtr, Request, Response, Wrappable, Wrapped, ref_counted_ptr
};
use cef_ui_sys::{
    cef_callback_t, cef_request_t, cef_resource_handler_t, cef_resource_read_callback_t,
    cef_resource_skip_callback_t, cef_response_t, cef_string_t
};
use std::{ffi::c_int, mem::zeroed, os::raw::c_void};

pub trait ResourceHandlerCallbacks: Send + Sync + 'static {
    /// Open the response stream. To handle the request immediately set
    /// |handle_request| to true and return true. To decide at a later time set
    /// |handle_request| to false, return true, and execute |callback| to continue
    /// or cancel the request. To cancel the request immediately set
    /// |handle_request| to true and return false. This method will be called in
    /// sequence but not from a dedicated thread. For backwards compatibility set
    /// |handle_request| to false and return false and the ProcessRequest method
    /// will be called.
    fn open(&mut self, _request: Request, _handle_request: &mut bool, _callback: Callback) -> bool {
        false
    }

    /// Begin processing the request. To handle the request return true and call
    /// CefCallback::Continue() once the response header information is available
    /// (CefCallback::Continue() can also be called from inside this method if
    /// header information is available immediately). To cancel the request return
    /// false.
    fn process_request(&mut self, _request: Request, _callback: Callback) -> bool {
        false
    }

    /// Retrieve response header information. If the response length is not known
    /// set |response_length| to -1 and ReadResponse() will be called until it
    /// returns false. If the response length is known set |response_length|
    /// to a positive value and ReadResponse() will be called until it returns
    /// false or the specified number of bytes have been read. Use the |response|
    /// object to set the mime type, http status code and other optional header
    /// values. To redirect the request to a new URL set |redirectUrl| to the new
    /// URL. |redirectUrl| can be either a relative or fully qualified URL.
    /// It is also possible to set |response| to a redirect http status code
    /// and pass the new URL via a Location header. Likewise with |redirectUrl| it
    /// is valid to set a relative or fully qualified URL as the Location header
    /// value. If an error occured while setting up the request you can call
    /// SetError() on |response| to indicate the error condition.
    fn get_response_headers(
        &mut self,
        _response: Response,
        _response_length: &mut i64,
        _redirect_url: &mut String
    ) {
    }

    /// Skip response data when requested by a Range header. Skip over and discard
    /// |bytes_to_skip| bytes of response data. If data is available immediately
    /// set |bytes_skipped| to the number of bytes skipped and return true. To
    /// read the data at a later time set |bytes_skipped| to 0, return true and
    /// execute |callback| when the data is available. To indicate failure set
    /// |bytes_skipped| to < 0 (e.g. -2 for ERR_FAILED) and return false. This
    /// method will be called in sequence but not from a dedicated thread.
    fn skip(
        &mut self,
        _bytes_to_skip: i64,
        _bytes_skipped: &mut i64,
        _callback: *mut cef_resource_skip_callback_t
    ) -> bool {
        false
    }

    /// Read response data. If data is available immediately copy up to
    /// |bytes_to_read| bytes into |data_out|, set |bytes_read| to the number of
    /// bytes copied, and return true. To read the data at a later time keep a
    /// pointer to |data_out|, set |bytes_read| to 0, return true and execute
    /// |callback| when the data is available (|data_out| will remain valid until
    /// the callback is executed). To indicate response completion set
    /// |bytes_read| to 0 and return false. To indicate failure set |bytes_read|
    /// to < 0 (e.g. -2 for ERR_FAILED) and return false. This method will be
    /// called in sequence but not from a dedicated thread. For backwards
    /// compatibility set |bytes_read| to -1 and return false and the ReadResponse
    /// method will be called.
    fn read(
        &mut self,
        _data_out: *mut c_void,
        _bytes_to_read: c_int,
        bytes_read: &mut c_int,
        _callback: *mut cef_resource_read_callback_t
    ) -> bool {
        *bytes_read = -1;
        false
    }

    /// Read response data. If data is available immediately copy up to
    /// |bytes_to_read| bytes into |data_out|, set |bytes_read| to the number of
    /// bytes copied, and return true. To read the data at a later time set
    /// |bytes_read| to 0, return true and call CefCallback::Continue() when the
    /// data is available. To indicate response completion return false.
    fn read_response(
        &mut self,
        _data_out: *mut c_void,
        _bytes_to_read: c_int,
        bytes_read: &mut c_int,
        _callback: Callback
    ) -> bool {
        *bytes_read = 0;
        false
    }

    /// Request processing has been canceled.
    fn cancel(&mut self) {}
}

ref_counted_ptr!(ResourceHandler, cef_resource_handler_t);

impl ResourceHandler {
    pub fn new<C: ResourceHandlerCallbacks>(delegate: C) -> Self {
        Self(ResourceHandlerWrapper::new(delegate).wrap())
    }
}

// Class used to implement a custom request handler interface. The methods of
// this class will be called on the IO thread unless otherwise indicated.
struct ResourceHandlerWrapper(Box<dyn ResourceHandlerCallbacks>);

/// Class used to implement a custom request handler interface. The methods of
/// this class will be called on the IO thread unless otherwise indicated.
impl ResourceHandlerWrapper {
    pub fn new<C: ResourceHandlerCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }

    unsafe extern "C" fn c_open(
        this: *mut cef_resource_handler_t,
        request: *mut cef_request_t,
        handle_request: *mut c_int,
        callback: *mut cef_callback_t
    ) -> c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let request = Request::from_ptr_unchecked(request);
        let mut handle_req = *handle_request != 0;
        let callback = Callback::from_ptr_unchecked(callback);

        let result = this
            .0
            .open(request, &mut handle_req, callback);
        *handle_request = if handle_req { 1 } else { 0 };
        result as c_int
    }

    unsafe extern "C" fn c_process_request(
        this: *mut cef_resource_handler_t,
        request: *mut cef_request_t,
        callback: *mut cef_callback_t
    ) -> c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let request = Request::from_ptr_unchecked(request);
        let callback = Callback::from_ptr_unchecked(callback);

        this.0
            .process_request(request, callback) as c_int
    }

    unsafe extern "C" fn c_get_response_headers(
        this: *mut cef_resource_handler_t,
        response: *mut cef_response_t,
        response_length: *mut i64,
        redirect_url: *mut cef_string_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let response = Response::from_ptr_unchecked(response);
        let mut resp_length = *response_length;
        let mut redirect: String = CefString::from_ptr_unchecked(redirect_url).into();

        this.0
            .get_response_headers(response, &mut resp_length, &mut redirect);

        *response_length = resp_length;
        let redirect_cef = CefString::new(&redirect);
        std::ptr::copy_nonoverlapping(redirect_cef.as_ptr(), redirect_url, 1);
    }

    unsafe extern "C" fn c_skip(
        this: *mut cef_resource_handler_t,
        bytes_to_skip: i64,
        bytes_skipped: *mut i64,
        callback: *mut cef_resource_skip_callback_t
    ) -> c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let mut skipped = *bytes_skipped;

        let result = this
            .0
            .skip(bytes_to_skip, &mut skipped, callback);
        *bytes_skipped = skipped;
        result as c_int
    }

    unsafe extern "C" fn c_read(
        this: *mut cef_resource_handler_t,
        data_out: *mut c_void,
        bytes_to_read: c_int,
        bytes_read: *mut c_int,
        callback: *mut cef_resource_read_callback_t
    ) -> c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let mut read_count = *bytes_read;

        let result = this
            .0
            .read(data_out, bytes_to_read, &mut read_count, callback);
        *bytes_read = read_count;
        result as c_int
    }

    unsafe extern "C" fn c_read_response(
        this: *mut cef_resource_handler_t,
        data_out: *mut c_void,
        bytes_to_read: c_int,
        bytes_read: *mut c_int,
        callback: *mut cef_callback_t
    ) -> c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let mut read_count = *bytes_read;
        let callback = Callback::from_ptr_unchecked(callback);

        let result = this
            .0
            .read_response(data_out, bytes_to_read, &mut read_count, callback);
        *bytes_read = read_count;
        result as c_int
    }

    unsafe extern "C" fn c_cancel(this: *mut cef_resource_handler_t) {
        let this: &mut Self = Wrapped::wrappable(this);
        this.0.cancel();
    }
}

impl Wrappable for ResourceHandlerWrapper {
    type Cef = cef_resource_handler_t;

    fn wrap(self) -> RefCountedPtr<cef_resource_handler_t> {
        RefCountedPtr::wrap(
            cef_resource_handler_t {
                base:                 unsafe { zeroed() },
                open:                 Some(Self::c_open),
                process_request:      Some(Self::c_process_request),
                get_response_headers: Some(Self::c_get_response_headers),
                skip:                 Some(Self::c_skip),
                read:                 Some(Self::c_read),
                read_response:        Some(Self::c_read_response),
                cancel:               Some(Self::c_cancel)
            },
            self
        )
    }
}
