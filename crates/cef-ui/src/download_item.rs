use crate::{CefString, ref_counted_ptr, try_c};
use anyhow::Result;
use cef_ui_sys::{cef_basetime_t, cef_download_interrupt_reason_t, cef_download_item_t};

// Structure used to represent a download item.
ref_counted_ptr!(DownloadItem, cef_download_item_t);

impl DownloadItem {
    /// Returns true if this object is valid. Do not call any other methods if
    /// this function returns false.
    pub fn is_valid(&self) -> Result<bool> {
        try_c!(self, is_valid, { Ok(is_valid(self.as_ptr()) != 0) })
    }

    /// Returns true if the download is in progress.
    pub fn is_in_progress(&self) -> Result<bool> {
        try_c!(self, is_in_progress, {
            Ok(is_in_progress(self.as_ptr()) != 0)
        })
    }

    /// Returns true if the download is complete.
    pub fn is_complete(&self) -> Result<bool> {
        try_c!(self, is_complete, { Ok(is_complete(self.as_ptr()) != 0) })
    }

    /// Returns true if the download has been canceled.
    pub fn is_canceled(&self) -> Result<bool> {
        try_c!(self, is_canceled, { Ok(is_canceled(self.as_ptr()) != 0) })
    }

    /// Returns true if the download has been interrupted.
    pub fn is_interrupted(&self) -> Result<bool> {
        try_c!(self, is_interrupted, {
            Ok(is_interrupted(self.as_ptr()) != 0)
        })
    }

    /// Returns the most recent interrupt reason.
    pub fn get_interrupt_reason(&self) -> Result<cef_download_interrupt_reason_t> {
        try_c!(self, get_interrupt_reason, {
            Ok(get_interrupt_reason(self.as_ptr()))
        })
    }

    /// Returns a simple speed estimate in bytes/s.
    pub fn get_current_speed(&self) -> Result<i64> {
        try_c!(self, get_current_speed, {
            Ok(get_current_speed(self.as_ptr()))
        })
    }

    /// Returns the rough percent complete or -1 if the receive total size is
    /// unknown.
    pub fn get_percent_complete(&self) -> Result<i32> {
        try_c!(self, get_percent_complete, {
            Ok(get_percent_complete(self.as_ptr()))
        })
    }

    /// Returns the total number of bytes.
    pub fn get_total_bytes(&self) -> Result<i64> {
        try_c!(self, get_total_bytes, {
            Ok(get_total_bytes(self.as_ptr()))
        })
    }

    /// Returns the number of received bytes.
    pub fn get_received_bytes(&self) -> Result<i64> {
        try_c!(self, get_received_bytes, {
            Ok(get_received_bytes(self.as_ptr()))
        })
    }

    /// Returns the time that the download started.
    pub fn get_start_time(&self) -> Result<cef_basetime_t> {
        try_c!(self, get_start_time, { Ok(get_start_time(self.as_ptr())) })
    }

    /// Returns the time that the download ended.
    pub fn get_end_time(&self) -> Result<cef_basetime_t> {
        try_c!(self, get_end_time, { Ok(get_end_time(self.as_ptr())) })
    }

    /// Returns the full path to the downloaded or downloading file.
    pub fn get_full_path(&self) -> Result<String> {
        try_c!(self, get_full_path, {
            let cef_string = get_full_path(self.as_ptr());
            Ok(CefString::from_userfree_ptr(cef_string)
                .unwrap_or_default()
                .into())
        })
    }

    /// Returns the unique identifier for this download.
    pub fn get_id(&self) -> Result<u32> {
        try_c!(self, get_id, { Ok(get_id(self.as_ptr())) })
    }

    /// Returns the URL.
    pub fn get_url(&self) -> Result<String> {
        try_c!(self, get_url, {
            let cef_string = get_url(self.as_ptr());
            Ok(CefString::from_userfree_ptr(cef_string)
                .unwrap_or_default()
                .into())
        })
    }

    /// Returns the original URL before any redirections.
    pub fn get_original_url(&self) -> Result<String> {
        try_c!(self, get_original_url, {
            let cef_string = get_original_url(self.as_ptr());
            Ok(CefString::from_userfree_ptr(cef_string)
                .unwrap_or_default()
                .into())
        })
    }

    /// Returns the suggested file name.
    pub fn get_suggested_file_name(&self) -> Result<String> {
        try_c!(self, get_suggested_file_name, {
            let cef_string = get_suggested_file_name(self.as_ptr());
            Ok(CefString::from_userfree_ptr(cef_string)
                .unwrap_or_default()
                .into())
        })
    }

    /// Returns the content disposition.
    pub fn get_content_disposition(&self) -> Result<String> {
        try_c!(self, get_content_disposition, {
            let cef_string = get_content_disposition(self.as_ptr());
            Ok(CefString::from_userfree_ptr(cef_string)
                .unwrap_or_default()
                .into())
        })
    }

    /// Returns the mime type.
    pub fn get_mime_type(&self) -> Result<String> {
        try_c!(self, get_mime_type, {
            let cef_string = get_mime_type(self.as_ptr());
            Ok(CefString::from_userfree_ptr(cef_string)
                .unwrap_or_default()
                .into())
        })
    }
}
