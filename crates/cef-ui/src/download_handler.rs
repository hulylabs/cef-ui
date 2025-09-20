use crate::{
    Browser, CefString, DownloadItem, RefCountedPtr, Wrappable, Wrapped, ref_counted_ptr, try_c
};
use anyhow::Result;
use cef_ui_sys::{
    cef_before_download_callback_t, cef_browser_t, cef_download_handler_t,
    cef_download_item_callback_t, cef_download_item_t, cef_string_t
};

use std::{ffi::c_int, mem::zeroed};

// Callback interface used to asynchronously continue a download.
ref_counted_ptr!(BeforeDownloadCallback, cef_before_download_callback_t);

impl BeforeDownloadCallback {
    /// Call to continue the download. Set |download_path| to the full file path
    /// for the download including the file name or leave blank to use the
    /// suggested name and the default temp directory. Set |show_dialog| to true
    /// if you do wish to show the default "Save As" dialog.
    pub fn continue_download(&self, download_path: Option<&str>, show_dialog: bool) -> Result<()> {
        try_c!(self, cont, {
            let download_path = download_path.unwrap_or("");
            let download_path = CefString::new(download_path);
            Ok(cont(
                self.as_ptr(),
                download_path.as_ptr(),
                show_dialog as c_int
            ))
        })
    }
}

// Callback interface used to asynchronously cancel a download.
ref_counted_ptr!(DownloadItemCallback, cef_download_item_callback_t);

impl DownloadItemCallback {
    /// Call to cancel the download.
    pub fn cancel(&self) -> Result<()> {
        try_c!(self, cancel, { Ok(cancel(self.as_ptr())) })
    }

    /// Call to pause the download.
    pub fn pause(&self) -> Result<()> {
        try_c!(self, pause, { Ok(pause(self.as_ptr())) })
    }

    /// Call to resume the download.
    pub fn resume(&self) -> Result<()> {
        try_c!(self, resume, { Ok(resume(self.as_ptr())) })
    }
}

/// Trait used to handle file downloads. The methods of this trait will be called
/// on the browser process UI thread.
pub trait DownloadHandlerCallbacks: Send + Sync + 'static {
    /// Called before a download begins in response to a user-initiated action
    /// (e.g. alt + link click or link click that returns a `Content-Disposition:
    /// attachment` response from the server). |url| is the target download URL and
    /// |request_method| is the target method (GET, POST, etc). Return true to
    /// proceed with the download or false to cancel the download.
    fn can_download(&mut self, _browser: Browser, _url: &str, _request_method: &str) -> bool {
        true
    }

    /// Called before a download begins. |suggested_name| is the suggested name
    /// for the download file. Return true and execute |callback| either
    /// asynchronously or in this method to continue or cancel the download.
    /// Return false to proceed with default handling (cancel with Alloy style,
    /// download shelf with Chrome style). Do not keep a reference to
    /// |download_item| outside of this method.
    fn on_before_download(
        &mut self,
        _browser: Browser,
        _download_item: DownloadItem,
        _suggested_name: &str,
        _callback: BeforeDownloadCallback
    ) -> bool {
        false
    }

    /// Called when a download's status or progress information has been updated.
    /// This may be called multiple times before and after on_before_download().
    /// Execute |callback| either asynchronously or in this method to cancel the
    /// download if desired. Do not keep a reference to |download_item| outside of
    /// this method.
    fn on_download_updated(
        &mut self,
        _browser: Browser,
        _download_item: DownloadItem,
        _callback: DownloadItemCallback
    ) {
    }
}

ref_counted_ptr!(DownloadHandler, cef_download_handler_t);

impl DownloadHandler {
    pub fn new<C: DownloadHandlerCallbacks>(delegate: C) -> Self {
        Self(DownloadHandlerWrapper::new(delegate).wrap())
    }
}

struct DownloadHandlerWrapper(Box<dyn DownloadHandlerCallbacks>);

impl DownloadHandlerWrapper {
    pub fn new<C: DownloadHandlerCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }

    unsafe extern "C" fn c_can_download(
        this: *mut cef_download_handler_t,
        browser: *mut cef_browser_t,
        url: *const cef_string_t,
        request_method: *const cef_string_t
    ) -> ::std::os::raw::c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let url: String = CefString::from_ptr_unchecked(url).into();
        let request_method: String = CefString::from_ptr_unchecked(request_method).into();

        this.0
            .can_download(browser, &url, &request_method) as ::std::os::raw::c_int
    }

    unsafe extern "C" fn c_on_before_download(
        this: *mut cef_download_handler_t,
        browser: *mut cef_browser_t,
        download_item: *mut cef_download_item_t,
        suggested_name: *const cef_string_t,
        callback: *mut cef_before_download_callback_t
    ) -> ::std::os::raw::c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let download_item = DownloadItem::from_ptr_unchecked(download_item);
        let suggested_name: String = CefString::from_ptr_unchecked(suggested_name).into();
        let callback = BeforeDownloadCallback::from_ptr_unchecked(callback);

        this.0
            .on_before_download(browser, download_item, &suggested_name, callback)
            as ::std::os::raw::c_int
    }

    unsafe extern "C" fn c_on_download_updated(
        this: *mut cef_download_handler_t,
        browser: *mut cef_browser_t,
        download_item: *mut cef_download_item_t,
        callback: *mut cef_download_item_callback_t
    ) {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let download_item = DownloadItem::from_ptr_unchecked(download_item);
        let callback = DownloadItemCallback::from_ptr_unchecked(callback);

        this.0
            .on_download_updated(browser, download_item, callback);
    }
}

impl Wrappable for DownloadHandlerWrapper {
    type Cef = cef_download_handler_t;

    fn wrap(self) -> RefCountedPtr<cef_download_handler_t> {
        RefCountedPtr::wrap(
            cef_download_handler_t {
                base:                unsafe { zeroed() },
                can_download:        Some(Self::c_can_download),
                on_before_download:  Some(Self::c_on_before_download),
                on_download_updated: Some(Self::c_on_download_updated)
            },
            self
        )
    }
}
