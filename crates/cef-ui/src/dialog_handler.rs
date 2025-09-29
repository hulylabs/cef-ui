use crate::{
    Browser, CefString, CefStringList, RefCountedPtr, Wrappable, Wrapped, ref_counted_ptr, try_c
};
use anyhow::Result;
use cef_ui_sys::{
    cef_browser_t, cef_dialog_handler_t, cef_file_dialog_callback_t, cef_file_dialog_mode_t,
    cef_string_list_t, cef_string_t
};

use std::mem::zeroed;

// Callback interface for asynchronous continuation of file dialog requests.
ref_counted_ptr!(FileDialogCallback, cef_file_dialog_callback_t);

impl FileDialogCallback {
    /// Continue the file selection. |file_paths| should be a single value or a
    /// list of values depending on the dialog mode. An empty |file_paths| value
    /// is treated the same as calling Cancel().
    pub fn continue_dialog(&self, file_paths: Vec<String>) -> Result<()> {
        try_c!(self, cont, {
            let mut list = CefStringList::from(file_paths);
            let result = cont(self.as_ptr(), list.as_mut_ptr());
            Ok(result)
        })
    }

    /// Cancel the file selection.
    pub fn cancel(&self) -> Result<()> {
        try_c!(self, cancel, { Ok(cancel(self.as_ptr())) })
    }
}

/// File dialog mode enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileDialogMode {
    /// Requires that the file exists before allowing the user to pick it.
    Open,
    /// Like Open, but allows picking multiple files to open.
    OpenMultiple,
    /// Like Open, but selects a folder to open.
    OpenFolder,
    /// Allows picking a file that doesn't exist yet.
    Save
}

impl From<cef_file_dialog_mode_t> for FileDialogMode {
    fn from(mode: cef_file_dialog_mode_t) -> Self {
        match mode {
            cef_ui_sys::cef_file_dialog_mode_t::FILE_DIALOG_OPEN => FileDialogMode::Open,
            cef_ui_sys::cef_file_dialog_mode_t::FILE_DIALOG_OPEN_MULTIPLE => {
                FileDialogMode::OpenMultiple
            },
            cef_ui_sys::cef_file_dialog_mode_t::FILE_DIALOG_OPEN_FOLDER => {
                FileDialogMode::OpenFolder
            },
            cef_ui_sys::cef_file_dialog_mode_t::FILE_DIALOG_SAVE => FileDialogMode::Save
        }
    }
}

impl From<FileDialogMode> for cef_file_dialog_mode_t {
    fn from(mode: FileDialogMode) -> Self {
        match mode {
            FileDialogMode::Open => cef_ui_sys::cef_file_dialog_mode_t::FILE_DIALOG_OPEN,
            FileDialogMode::OpenMultiple => {
                cef_ui_sys::cef_file_dialog_mode_t::FILE_DIALOG_OPEN_MULTIPLE
            },
            FileDialogMode::OpenFolder => {
                cef_ui_sys::cef_file_dialog_mode_t::FILE_DIALOG_OPEN_FOLDER
            },
            FileDialogMode::Save => cef_ui_sys::cef_file_dialog_mode_t::FILE_DIALOG_SAVE
        }
    }
}

/// Implement this interface to handle dialog events. The methods of this class
/// will be called on the browser process UI thread.
pub trait DialogHandlerCallbacks: Send + Sync + 'static {
    /// Called to run a file chooser dialog. |mode| represents the type of dialog
    /// to display. |title| to the title to be used for the dialog and may be
    /// empty to show the default title ("Open" or "Save" depending on the mode).
    /// |default_file_path| is the path with optional directory and/or file name
    /// component that should be initially selected in the dialog.
    /// |accept_filters| are used to restrict the selectable file types and may be
    /// any combination of valid lower-cased MIME types (e.g. "text/*" or
    /// "image/*") and individual file extensions (e.g. ".txt" or ".png").
    /// |accept_extensions| provides the semicolon-delimited expansion of MIME
    /// types to file extensions (if known, or empty string otherwise).
    /// |accept_descriptions| provides the descriptions for MIME types (if known,
    /// or empty string otherwise). For example, the "image/*" mime type might
    /// have extensions ".png;.jpg;.bmp;..." and description "Image Files".
    /// |accept_filters|, |accept_extensions| and |accept_descriptions| will all
    /// be the same size. To display a custom dialog return true and execute
    /// |callback| either inline or at a later time. To display the default dialog
    /// return false. If this method returns false it may be called an additional
    /// time for the same dialog (both before and after MIME type expansion).
    fn on_file_dialog(
        &mut self,
        _browser: Browser,
        _mode: FileDialogMode,
        _title: String,
        _default_file_path: String,
        _accept_filters: Vec<String>,
        _accept_extensions: Vec<String>,
        _accept_descriptions: Vec<String>,
        _callback: FileDialogCallback
    ) -> bool {
        false
    }
}

// Implement this interface to handle dialog events. The methods of this class
// will be called on the browser process UI thread.
ref_counted_ptr!(DialogHandler, cef_dialog_handler_t);

impl DialogHandler {
    pub fn new<C: DialogHandlerCallbacks>(delegate: C) -> Self {
        Self(DialogHandlerWrapper::new(delegate).wrap())
    }
}

struct DialogHandlerWrapper(Box<dyn DialogHandlerCallbacks>);

impl DialogHandlerWrapper {
    pub fn new<C: DialogHandlerCallbacks>(delegate: C) -> Self {
        Self(Box::new(delegate))
    }

    unsafe extern "C" fn c_on_file_dialog(
        this: *mut cef_dialog_handler_t,
        browser: *mut cef_browser_t,
        mode: cef_file_dialog_mode_t,
        title: *const cef_string_t,
        default_file_path: *const cef_string_t,
        accept_filters: cef_string_list_t,
        accept_extensions: cef_string_list_t,
        accept_descriptions: cef_string_list_t,
        callback: *mut cef_file_dialog_callback_t
    ) -> ::std::os::raw::c_int {
        let this: &mut Self = Wrapped::wrappable(this);
        let browser = Browser::from_ptr_unchecked(browser);
        let mode = FileDialogMode::from(mode);
        let title: String = CefString::from_ptr_unchecked(title).into();
        let default_file_path: String = CefString::from_ptr_unchecked(default_file_path).into();

        // Convert string lists to Vec<String>
        let accept_filters = CefStringList::from_ptr_unchecked(accept_filters);
        let accept_extensions = CefStringList::from_ptr_unchecked(accept_extensions);
        let accept_descriptions = CefStringList::from_ptr_unchecked(accept_descriptions);

        let callback = FileDialogCallback::from_ptr_unchecked(callback);

        this.0.on_file_dialog(
            browser,
            mode,
            title,
            default_file_path,
            accept_filters.into(),
            accept_extensions.into(),
            accept_descriptions.into(),
            callback
        ) as ::std::os::raw::c_int
    }
}

impl Wrappable for DialogHandlerWrapper {
    type Cef = cef_dialog_handler_t;

    fn wrap(self) -> RefCountedPtr<cef_dialog_handler_t> {
        RefCountedPtr::wrap(
            cef_dialog_handler_t {
                base:           unsafe { zeroed() },
                on_file_dialog: Some(Self::c_on_file_dialog)
            },
            self
        )
    }
}
